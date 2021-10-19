use crate::builder::NodeBuilder;
use crate::style::{self, FONT_SIZE};
use crate::widget::pad;
use iced_wgpu::Renderer;
use iced_winit::{
    alignment, Alignment, Column, Container, Element, Length, Point, Row, Rule, Space, Text,
};
pub use naga::{ScalarKind as Scalar, VectorSize};

pub mod input;
pub mod master;
pub mod math;

mod edge;
mod port;

pub use self::edge::{Edge, Input, Output, Pending};
pub use self::port::{PortId, PortState};

slotmap::new_key_type! { pub struct NodeId; }

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Scalar(Scalar),
    Vector(Scalar, VectorSize),
    Matrix(Scalar, VectorSize, VectorSize),
}

impl Type {
    pub const V2F: Self = Self::Vector(Scalar::Float, VectorSize::Bi);
    pub const V3F: Self = Self::Vector(Scalar::Float, VectorSize::Tri);
    pub const V4F: Self = Self::Vector(Scalar::Float, VectorSize::Quad);
}

pub trait BoxedNode {
    fn boxed() -> Box<dyn Node>;
}

impl<T: Node + Default> BoxedNode for T {
    fn boxed() -> Box<dyn Node> {
        Box::new(Self::default())
    }
}

pub type Dynamic = Box<dyn DynMessage>;

pub trait DynMessage: downcast_rs::Downcast + std::fmt::Debug + Send + Sync {
    fn box_clone(&self) -> Box<dyn DynMessage>;
}

downcast_rs::impl_downcast!(DynMessage);

impl<T: std::any::Any + std::fmt::Debug + Clone + Send + Sync> DynMessage for T {
    fn box_clone(&self) -> Box<dyn DynMessage> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn DynMessage> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

pub fn downcast_message<T: DynMessage + Clone>(message: Box<dyn DynMessage>) -> Option<T> {
    message.downcast_ref().cloned()
}

pub fn upcast_message(message: impl DynMessage + Clone) -> Box<dyn DynMessage> {
    Box::new(message)
}

pub enum Event {
    Dynamic(Box<dyn DynMessage>),
    Input(PortId, Option<Output>),
    Output(PortId, Option<Input>),
}

#[derive(Debug)]
pub struct GenError;

pub struct NodeDescriptor<'a> {
    pub label: &'a str,
    pub width: u16,
    pub inputs: &'a [(&'a str, Type)],
    pub outputs: &'a [(&'a str, Type)],
}

pub trait Node: std::fmt::Debug + 'static + downcast_rs::Downcast + NodeBuilder {
    fn desc(&self) -> NodeDescriptor<'_>;

    fn update(&mut self, event: Event) {
        match event {
            Event::Dynamic(_) => log::info!("node internal event"),
            Event::Input(port, remote) => log::info!("node input event: {:?} {:?}", port, remote),
            Event::Output(port, remote) => log::info!("node output event: {:?} {:?}", port, remote),
        }
    }

    fn view(
        &mut self,
        _node: NodeId,
    ) -> iced_native::Element<Box<dyn DynMessage>, iced_wgpu::Renderer> {
        iced_native::Space::new(iced_native::Length::Shrink, iced_native::Length::Shrink).into()
    }
}

downcast_rs::impl_downcast!(Node);

#[derive(Default)]
pub struct NodeMap(slotmap::SlotMap<NodeId, NodeWidget>);

impl std::ops::Deref for NodeMap {
    type Target = slotmap::SlotMap<NodeId, NodeWidget>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for NodeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Index<NodeId> for NodeMap {
    type Output = NodeWidget;
    fn index(&self, index: NodeId) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<NodeId> for NodeMap {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl std::ops::Index<Output> for NodeMap {
    type Output = NodeWidget;
    fn index(&self, index: Output) -> &Self::Output {
        &self.0[index.node()]
    }
}

impl std::ops::IndexMut<Output> for NodeMap {
    fn index_mut(&mut self, index: Output) -> &mut Self::Output {
        &mut self.0[index.node()]
    }
}

impl std::ops::Index<Input> for NodeMap {
    type Output = NodeWidget;
    fn index(&self, index: Input) -> &Self::Output {
        &self.0[index.node()]
    }
}

impl std::ops::IndexMut<Input> for NodeMap {
    fn index_mut(&mut self, index: Input) -> &mut Self::Output {
        &mut self.0[index.node()]
    }
}

impl crate::builder::NodeBuilder for NodeMap {
    fn expr(
        &self,
        function: &mut crate::builder::FunctionBuilder,
        output: Output,
    ) -> Option<naga::Handle<naga::Expression>> {
        self.0[output.node()].node.expr(function, output)
    }
}

impl ToString for NodeId {
    fn to_string(&self) -> String {
        let value = slotmap::Key::data(self).as_ffi();
        let idx = (value & 0xffff_ffff) as u32;
        let version = ((value >> 32) | 1) as u32; // Ensure version is odd.
        format!("{}v{}", idx, version)
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Dynamic(NodeId, Dynamic),
    Remove(NodeId),

    DragStart(NodeId),
    DragMove(Point),
    DragEnd(NodeId),

    StartEdge(Pending),
    CancelEdge,
}

pub struct NodeWidget {
    pub id: NodeId,
    pub position: Point,

    pub inputs: Vec<PortState>,
    pub outputs: Vec<PortState>,

    pub node: Box<dyn crate::node::Node>,

    title_state: pad::State,
    close: pad::State,

    xstate: [VectorDefaults; 2],
}

impl NodeWidget {
    pub fn new(id: NodeId, position: Point, node: impl Into<Box<dyn crate::node::Node>>) -> Self {
        let node = node.into();
        let desc = node.desc();
        Self {
            id,
            position,
            inputs: desc
                .inputs
                .iter()
                .map(|(name, _)| PortState::new(name))
                .collect(),
            outputs: desc
                .outputs
                .iter()
                .map(|(name, _)| PortState::new(name))
                .collect(),
            node,
            title_state: Default::default(),
            close: Default::default(),
            xstate: Default::default(),
        }
    }

    pub fn event(&mut self, event: Event) {
        self.node.update(event)
    }

    pub fn widget(&mut self) -> Element<Message, Renderer> {
        let node = self.id;

        fn grap_pad<'a>(
            node: NodeId,
            state: &'a mut pad::State,
            content: impl Into<Element<'a, Message, Renderer>>,
        ) -> pad::Pad<Message> {
            pad::Pad::new(state, content)
                .padding([2, 0])
                .on_press(Message::DragStart(node))
                .on_release(Message::DragEnd(node))
                .interaction(iced_native::mouse::Interaction::Grab)
        }

        fn create_ports(
            node: NodeId,
            ports: &mut [PortState],
            pending: impl Fn(NodeId, PortId) -> Pending,
        ) -> Element<Message, Renderer> {
            if ports.is_empty() {
                Space::new(Length::Shrink, Length::Shrink).into()
            } else {
                ports
                    .iter_mut()
                    .enumerate()
                    .fold(
                        Column::new().width(Length::Fill).spacing(2).padding([4, 0]),
                        |inputs, (index, state)| {
                            inputs.push(state.view(pending(node, PortId(index))))
                        },
                    )
                    .into()
            }
        }

        let title = {
            let title = text_left(self.node.desc().label).width(Length::Fill);
            let title = grap_pad(self.id, &mut self.title_state, title)
                .width(Length::Fill)
                .padding([0, 4]);

            let close = text_center("×").size(16).width(Length::Units(FONT_SIZE));
            let close = pad::Pad::new(&mut self.close, close)
                .padding([4, 0])
                .on_release(Message::Remove(self.id));

            Row::new()
                .width(Length::Fill)
                .align_items(Alignment::Center)
                .push(title)
                .push(close)
        };

        let width = self.node.desc().width;

        let title = Container::new(title).style(style::Node);
        let rule = Rule::horizontal(1).style(style::Transparent);

        let body = {
            let inputs = create_ports(node, &mut self.inputs, Pending::input);
            let outputs = create_ports(node, &mut self.outputs, Pending::output);

            let io = Row::new().push(inputs).push(outputs);
            let controls = self.node.view(node).map(move |m| Message::Dynamic(node, m));

            Container::new(Column::new().push(io).push(controls)).style(style::Node)
        };

        let inner = Column::new().push(title).push(rule).push(body);

        let node = Container::new(inner)
            .style(style::NodeBorder)
            .width(Length::Units(width))
            .height(Length::Shrink)
            .padding(3);

        let defaults = {
            let labels = Column::new()
                .align_items(Alignment::End)
                .width(Length::Shrink)
                .spacing(1)
                .padding([2, 0]);

            let mut n = 0;
            let labels = self.xstate.iter_mut().fold(labels, move |labels, state| {
                n += 1;
                labels.push(state.view(n))
            });

            let title = Space::new(Length::Shrink, Length::Units(25));
            Column::new().padding(3).push(title).push(labels)
        };

        Row::new()
            .push(defaults)
            .push(node)
            .width(Length::Units(width * 3))
            .into()
    }
}

#[derive(Default)]
struct VectorDefaults {
    state: [iced_native::widget::text_input::State; 4],
}

impl VectorDefaults {
    fn view(&mut self, n: usize) -> Element<Message, Renderer> {
        let mut row = Row::new().align_items(Alignment::Start);

        for (state, label) in self.state.iter_mut().zip(&["x", "y", "z", "w"]).take(n) {
            let label = text_center(label)
                .vertical_alignment(alignment::Vertical::Top)
                .size(style::FONT_SIZE - 2)
                .width(Length::Units(14))
                .height(Length::Units(14));

            let input = style::Node::input(state, "", "0.0", |_| {
                Message::Dynamic(NodeId::default(), Box::new(()))
            })
            .width(Length::Units(style::FONT_SIZE));

            row = row.push(label).push(input);
        }

        Container::new(row).style(style::Node).padding(1).into()
    }
}

fn text_center(label: &str) -> Text<Renderer> {
    Text::new(label)
        .size(FONT_SIZE)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
}

fn text_left(label: &str) -> Text<Renderer> {
    Text::new(label)
        .size(FONT_SIZE)
        .horizontal_alignment(alignment::Horizontal::Left)
        .vertical_alignment(alignment::Vertical::Center)
}
