use super::node::NodeId;
use iced_graphics::canvas::{Cursor, Frame, Geometry, Path, Stroke};
use iced_winit::{Point, Rectangle, Vector};
use slotmap::Key as _;
use std::fmt;

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PortId(pub usize);

impl ToString for PortId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Connection {
    pub node: NodeId,
    pub port: PortId,
    pub position: Point,
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}::{} {} {})",
            self.node.to_string(),
            self.port.to_string(),
            self.position.x,
            self.position.y
        )
    }
}

impl Connection {
    pub const fn new(node: NodeId, port: PortId, position: Point) -> Self {
        Self {
            node,
            port,
            position,
        }
    }

    pub fn eq_node_port(&self, other: &Self) -> bool {
        self.node == other.node && self.port == other.port
    }

    pub fn start(node: NodeId, port: PortId) -> Self {
        Self::new(node, port, Point::ORIGIN)
    }

    pub fn pending(position: Point) -> Self {
        Self::new(NodeId::null(), PortId(usize::MAX), position)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub output: Connection,
    pub input: Connection,
}

impl Edge {
    pub fn new(from: Pending, to: Pending) -> Option<Self> {
        match (from, to) {
            (Pending::Input(input), Pending::Output(output)) => Some(Self { output, input }),
            (Pending::Output(output), Pending::Input(input)) => Some(Self { output, input }),
            _ => None,
        }
    }

    pub fn not_same_node(&self) -> bool {
        self.output.node != self.input.node
    }

    pub fn eq_node_port(&self, other: &Self) -> bool {
        self.input.eq_node_port(&other.input) && self.output.eq_node_port(&other.output)
    }

    pub fn translate(mut self, offset: Vector) -> Self {
        self.output.position = self.output.position + offset;
        self.input.position = self.input.position + offset;
        self
    }

    fn reverse(self) -> Self {
        Self {
            output: self.input,
            input: self.output,
        }
    }

    pub fn draw(self, frame: &mut Frame) {
        Self::draw_all(&[self], frame);
    }

    pub fn draw_all(curves: &[Self], frame: &mut Frame) {
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
                .with_width(1.0),
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Pending {
    Input(Connection),
    Output(Connection),
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
        Self::Input(Connection::start(node, port))
    }

    pub fn output(node: NodeId, port: PortId) -> Self {
        Self::Output(Connection::start(node, port))
    }

    pub fn translate(mut self, offset: Vector) -> Self {
        match &mut self {
            Self::Input(conn) | Self::Output(conn) => conn.position = conn.position + offset,
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
            let to = Connection::pending(to);
            match self {
                Self::Input(from) => Edge {
                    output: from,
                    input: to,
                }
                .reverse(),
                Self::Output(from) => Edge {
                    output: from,
                    input: to,
                },
            }
            .draw(&mut frame)
        }

        frame.into_geometry()
    }
}
