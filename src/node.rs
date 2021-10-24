use crate::builder::NodeBuilder;
use crate::style::{self, FONT_SIZE};
use crate::widget::pad;
use iced_wgpu::Renderer;
use iced_winit::{
    alignment, Alignment, Column, Container, Element, Length, Point, Row, Rule, Space, Text,
};
use naga::{valid::ShaderStages, ScalarKind, VectorSize};

pub mod default;
pub mod input;
pub mod master;
pub mod math;

mod edge;
mod port;

pub use self::edge::{Edge, Input, InputMarker, Output, OutputMarker, Pending, Slot};
pub use self::port::PortId;

slotmap::new_key_type! { pub struct NodeId; }

struct SlotType {
    stages: ShaderStages,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Scalar(ScalarKind),
    Vector(ScalarKind, VectorSize),
    Matrix(ScalarKind, VectorSize, VectorSize),
}

impl Type {
    pub const V2F: Self = Self::Vector(ScalarKind::Float, VectorSize::Bi);
    pub const V3F: Self = Self::Vector(ScalarKind::Float, VectorSize::Tri);
    pub const V4F: Self = Self::Vector(ScalarKind::Float, VectorSize::Quad);
}

pub trait BoxedNode {
    fn boxed() -> Box<dyn Node>;
}

impl<T: Node + Default> BoxedNode for T {
    fn boxed() -> Box<dyn Node> {
        Box::new(Self::default())
    }
}

pub type NodeView<'a> = Element<'a, Box<dyn DynMessage>, Renderer>;

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
    AttachInput(PortId, Option<Output>),
    AttachOutput(PortId, Option<Input>),
}

#[derive(Debug)]
pub struct GenError;

pub struct NodeDescriptor<'a> {
    label: &'a str,
    width: u16,
    inputs: &'a [(&'a str, Type)],
    outputs: &'a [(&'a str, Type)],
}

pub trait Node: std::fmt::Debug + 'static + downcast_rs::Downcast + NodeBuilder {
    fn desc(&self) -> NodeDescriptor<'_>;

    fn update(&mut self, event: Event) {
        match event {
            Event::Dynamic(_) => log::info!("node internal event"),
            Event::AttachInput(port, remote) => {
                log::info!("node input event: {:?} {:?}", port, remote)
            }
            Event::AttachOutput(port, remote) => {
                log::info!("node output event: {:?} {:?}", port, remote)
            }
        }
    }

    fn view(&mut self, _node: NodeId) -> NodeView {
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
    Dynamic(NodeId, Box<dyn DynMessage>),

    SetDefault(NodeId, PortId, [f64; 4]),

    Remove(NodeId),

    DragStart(NodeId),
    DragMove(Point),
    DragEnd(NodeId),

    StartEdge(Pending),
    CancelEdge,
}

pub struct NodeWidget {
    id: NodeId,
    position: Point,
    title: Title,
    width: u16,

    node: Box<dyn crate::node::Node>,
    ports: Ports,
    defaults: Vec<default::InputDefault>,

    inputs_remote: Vec<Option<Output>>,
    outputs_remote: Vec<Option<Input>>,
}

impl NodeWidget {
    pub fn new(id: NodeId, position: Point, node: impl Into<Box<dyn crate::node::Node>>) -> Self {
        let node = node.into();
        let desc = node.desc();

        let mut defaults = Vec::with_capacity(desc.inputs.len());
        for _ in 0..desc.inputs.len() {
            defaults.push(default::InputDefault::new([0.0; 4], 4));
        }

        Self {
            id,
            position,
            width: desc.width,
            title: Title::new(&desc.label),
            inputs_remote: vec![None; desc.inputs.len()],
            outputs_remote: vec![None; desc.outputs.len()],
            ports: Ports {
                inputs: desc
                    .inputs
                    .iter()
                    .map(|(name, _)| port::State::new(name))
                    .collect(),
                outputs: desc
                    .outputs
                    .iter()
                    .map(|(name, _)| port::State::new(name))
                    .collect(),
            },
            defaults,
            node,
        }
    }

    pub fn downcast_ref<T: Node>(&self) -> Option<&T> {
        self.node.downcast_ref::<T>()
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    pub fn set_port_default(&mut self, port: PortId, value: [f64; 4]) {
        self.defaults[port.0].value = value;
    }

    pub fn inputs(&self) -> impl Iterator<Item = (PortId, &port::State)> + '_ {
        self.ports
            .inputs
            .iter()
            .enumerate()
            .map(|(i, port)| (PortId(i), port))
    }

    pub fn outputs(&self) -> impl Iterator<Item = (PortId, &port::State)> + '_ {
        self.ports
            .outputs
            .iter()
            .enumerate()
            .map(|(i, port)| (PortId(i), port))
    }

    pub fn event(&mut self, event: Event) {
        match event {
            Event::AttachInput(port, remote) => self.inputs_remote[port.0] = remote,
            Event::AttachOutput(port, remote) => self.outputs_remote[port.0] = remote,
            _ => (),
        }
        self.node.update(event)
    }

    pub fn widget(&mut self) -> Element<Message, Renderer> {
        let title = self.title.view(self.id);

        let title = Container::new(title).style(style::Node);
        let rule = Rule::horizontal(1).style(style::Transparent);

        let defaults =
            create_defaults_view(self.id, self.width, &mut self.defaults, &self.inputs_remote);

        let body = {
            let inputs = self.ports.input_view(self.id, &self.inputs_remote);
            let outputs = self.ports.output_view(self.id, &self.outputs_remote);

            let io = Row::new().push(inputs).push(outputs);
            let node = self.id;
            let controls = self.node.view(node).map(move |m| Message::Dynamic(node, m));

            Container::new(Column::new().push(io).push(controls)).style(style::Node)
        };

        let inner = Column::new().push(title).push(rule).push(body);

        let node = Container::new(inner)
            .style(style::NodeBorder)
            .width(Length::Units(self.width))
            .height(Length::Shrink)
            .padding(2);

        Row::new()
            .push(defaults)
            .push(node)
            .width(Length::Units(self.width * 3))
            .into()
    }
}

#[derive(Default)]
struct Title {
    label: String,
    title: pad::State,
    close: pad::State,
}

impl Title {
    pub fn new(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            title: pad::State::default(),
            close: pad::State::default(),
        }
    }

    pub fn view(&mut self, node: NodeId) -> Element<Message, Renderer> {
        let title = text_left(&self.label).width(Length::Fill);
        let title = grap_pad(node, &mut self.title, title)
            .width(Length::Fill)
            .padding([0, 4]);

        let close = text_center("×").size(16).width(Length::Units(FONT_SIZE));
        let close = pad::Pad::new(&mut self.close, close)
            .padding([4, 0])
            .on_release(Message::Remove(node));

        let row = Row::new().align_items(Alignment::Center);
        row.width(Length::Fill).push(title).push(close).into()
    }
}

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

struct Ports {
    inputs: Vec<port::State>,
    outputs: Vec<port::State>,
}

impl Ports {
    fn input_view(&self, node: NodeId, inputs: &[Option<Output>]) -> Element<Message, Renderer> {
        create_ports(node, &self.inputs, Pending::input, inputs)
    }

    fn output_view(&self, node: NodeId, outputs: &[Option<Input>]) -> Element<Message, Renderer> {
        create_ports(node, &self.outputs, Pending::output, outputs)
    }
}

fn create_ports<'a, T>(
    node: NodeId,
    ports: &'a [port::State],
    pending: impl Fn(NodeId, PortId) -> Pending,
    is_set: &[Option<Slot<T>>],
) -> Element<'a, Message, Renderer> {
    if ports.is_empty() {
        Space::new(Length::Shrink, Length::Shrink).into()
    } else {
        ports
            .iter()
            .zip(is_set.iter().map(Option::is_some))
            .enumerate()
            .fold(
                Column::new().width(Length::Fill).spacing(2).padding([4, 0]),
                |inputs, (index, (state, is_set))| {
                    inputs.push(state.view(pending(node, PortId(index)), is_set))
                },
            )
            .into()
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

fn create_defaults_view<'a>(
    node: NodeId,
    width: u16,
    defaults: &'a mut [default::InputDefault],
    is_set: &[Option<Output>],
) -> Element<'a, Message, Renderer> {
    let labels = Column::new()
        .align_items(Alignment::End)
        .width(Length::Shrink)
        .spacing(1)
        .padding([2, 0]);

    let labels = defaults.iter_mut().zip(is_set.iter().enumerate()).fold(
        labels,
        move |labels, (state, (port, input))| {
            labels.push(if input.is_some() {
                Element::from(Space::new(Length::Shrink, Length::Units(16)))
            } else {
                state.view(node, PortId(port))
            })
        },
    );

    let title = Space::new(Length::Shrink, Length::Units(25));
    Column::new()
        .align_items(Alignment::End)
        .width(Length::Units(width * 2))
        .padding(2)
        .push(title)
        .push(labels)
        .into()
}
