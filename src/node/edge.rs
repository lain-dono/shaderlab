use crate::node::{NodeId, PortId};
use iced_graphics::canvas::{Cursor, Frame, Geometry, Path, Stroke};
use iced_winit::{Point, Rectangle, Vector};
use std::{fmt, marker::PhantomData};

#[derive(Clone, Copy, Default, Debug)]
pub struct Slot<T> {
    node: NodeId,
    port: PortId,
    position: Point,
    marker: PhantomData<T>,
}

impl<T> PartialEq for Slot<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.node, self.port) == (other.node, other.port)
    }
}

impl<T> Slot<T> {
    #[inline]
    pub const fn new(node: NodeId, port: PortId) -> Self {
        Self {
            node,
            port,
            position: Point::ORIGIN,
            marker: PhantomData,
        }
    }

    #[inline]
    fn position(position: Point) -> Self {
        Self {
            node: NodeId::default(),
            port: PortId(usize::MAX),
            position,
            marker: PhantomData,
        }
    }

    #[inline]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    #[inline]
    pub const fn port(&self) -> PortId {
        self.port
    }
}

impl fmt::Display for Slot<OutputMarker> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Output({}::{} {}x{})",
            self.node.to_string(),
            self.port.to_string(),
            self.position.x,
            self.position.y
        )
    }
}

impl fmt::Display for Slot<InputMarker> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Input({}::{} {}x{})",
            self.node.to_string(),
            self.port.to_string(),
            self.position.x,
            self.position.y
        )
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[doc(hidden)]
pub struct InputMarker;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[doc(hidden)]
pub struct OutputMarker;

pub type Input = Slot<InputMarker>;
pub type Output = Slot<OutputMarker>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    output: Output,
    input: Input,
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Edge {{ {} -> {} }}", self.output, self.input,)
    }
}

impl Edge {
    pub fn new(from: Pending, to: Pending) -> Option<Self> {
        match (from, to) {
            (Pending::Input(input), Pending::Output(output)) => Some(Self { output, input }),
            (Pending::Output(output), Pending::Input(input)) => Some(Self { output, input }),
            _ => None,
        }
    }

    pub fn input(&self) -> Input {
        self.input
    }

    pub fn output(&self) -> Output {
        self.output
    }

    pub fn set_input_position(&mut self, position: Point) {
        self.input.position = position;
    }

    pub fn set_output_position(&mut self, position: Point) {
        self.output.position = position;
    }

    pub fn not_same_node(&self) -> bool {
        self.output.node != self.input.node
    }

    pub fn eq_node_port(&self, other: &Self) -> bool {
        self == other
    }

    pub fn has_node(&self, node: NodeId) -> bool {
        self.output.node == node || self.input.node == node
    }

    pub fn draw(frame: &mut Frame, curves: &[Self]) {
        let curves = Path::new(|p| {
            for curve in curves {
                let (from, to) = (curve.output.position, curve.input.position);
                let range = (from.distance(to) / 2.0).min(150.0);
                let control_a = from + Vector::new(range, 0.0);
                let control_b = to - Vector::new(range, 0.0);
                p.move_to(from);
                p.bezier_curve_to(control_a, control_b, to);
            }
        });

        frame.stroke(
            &curves,
            Stroke::default()
                .with_color(crate::style::PORT_COLOR)
                .with_width(1.5),
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Pending {
    Input(Slot<InputMarker>),
    Output(Slot<OutputMarker>),
}

impl fmt::Display for Pending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(c) => write!(f, "Input({})", c),
            Self::Output(c) => write!(f, "Output({})", c),
        }
    }
}

impl Pending {
    pub fn input(node: NodeId, port: PortId) -> Self {
        Self::Input(Slot::new(node, port))
    }

    pub fn output(node: NodeId, port: PortId) -> Self {
        Self::Output(Slot::new(node, port))
    }

    pub fn translate(mut self, offset: Vector) -> Self {
        match &mut self {
            Self::Input(conn) => conn.position = conn.position + offset,
            Self::Output(conn) => conn.position = conn.position + offset,
        }
        self
    }

    pub fn is_input(&self) -> bool {
        matches!(self, Self::Input(_))
    }

    pub fn is_output(&self) -> bool {
        matches!(self, Self::Output(_))
    }

    pub fn draw(self, bounds: Rectangle, cursor: Cursor) -> Geometry {
        let mut frame = Frame::new(bounds.size());

        if let Some(to) = cursor.position_in(&bounds) {
            let edge = match self {
                Self::Input(input) => Edge {
                    output: Slot::position(to),
                    input,
                },
                Self::Output(output) => Edge {
                    output,
                    input: Slot::position(to),
                },
            };
            Edge::draw(&mut frame, &[edge])
        }

        frame.into_geometry()
    }
}
