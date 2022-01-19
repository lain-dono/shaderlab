use crate::builder::FunctionBuilder;
use crate::style::{self, FONT_SIZE};
use crate::widget::pad;
use downcast_rs::Downcast;
use iced_graphics::{Column, Container, Row, Rule, Space};
use iced_native::widget::text::Text;
use iced_wgpu::Renderer;
use iced_winit::{alignment, Alignment, Element, Length, Point, Vector};
use naga::{Expression, Handle};
use std::ops::{Deref, DerefMut, Index, IndexMut};

pub mod default;
pub mod input;
pub mod master;
pub mod math;

mod edge;
mod port;

pub use self::edge::{Edge, Input, InputMarker, Output, OutputMarker, Pending, Slot};
pub use self::port::PortId;

pub type NodeElement<'a> = Element<'a, Message, Renderer>;

slotmap::new_key_type! { pub struct NodeId; }

impl ToString for NodeId {
    fn to_string(&self) -> String {
        let value = slotmap::Key::data(self).as_ffi();
        let idx = (value & 0xffff_ffff) as u32;
        let version = ((value >> 32) | 1) as u32; // Ensure version is odd.
        format!("{}v{}", idx, version)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Vector1,
    Vector2,
    Vector3,
    Vector4,
}

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
    SetDefault(PortId, [f64; 4]),
}

#[derive(Debug)]
pub struct GenError;

pub trait Node: Downcast + 'static {
    fn inputs(&self) -> &[port::State];
    fn outputs(&self) -> &[port::State];

    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>>;

    fn update(&mut self, event: Event);
    fn view(&mut self, _node: NodeId) -> Element<Message, Renderer>;
}

downcast_rs::impl_downcast!(Node);

#[derive(Default)]
pub struct NodeMap(slotmap::SlotMap<NodeId, (Point, Box<dyn Node>)>);

impl NodeMap {
    pub fn add(&mut self, position: Point, node: impl Node) {
        self.0.insert((position, Box::new(node)));
    }

    pub fn append_position(&mut self, node: NodeId, diff: Vector) {
        self.0[node].0 = self.0[node].0 + diff;
    }
}

impl Deref for NodeMap {
    type Target = slotmap::SlotMap<NodeId, (Point, Box<dyn Node>)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<NodeId> for NodeMap {
    type Output = dyn Node;
    fn index(&self, index: NodeId) -> &Self::Output {
        self.0[index].1.as_ref()
    }
}

impl IndexMut<NodeId> for NodeMap {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        self.0[index].1.as_mut()
    }
}

impl<T> Index<Slot<T>> for NodeMap {
    type Output = dyn Node;
    fn index(&self, index: Slot<T>) -> &Self::Output {
        self.0[index.node].1.as_ref()
    }
}

impl<T> IndexMut<Slot<T>> for NodeMap {
    fn index_mut(&mut self, index: Slot<T>) -> &mut Self::Output {
        self.0[index.node].1.as_mut()
    }
}

impl crate::builder::NodeBuilder for NodeMap {
    fn expr(
        &self,
        function: &mut crate::builder::FunctionBuilder,
        output: Output,
    ) -> Option<naga::Handle<naga::Expression>> {
        self.0[output.node].1.expr(function, output)
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
    title: Title,
    width: u16,

    ports: Ports,
    defaults: Vec<default::InputDefault>,

    inputs_remote: Vec<Option<Output>>,
    outputs_remote: Vec<Option<Input>>,
}

impl NodeWidget {
    pub fn new(label: &str, width: u16, inputs: &[(&str, Type)], outputs: &[(&str, Type)]) -> Self {
        let mut defaults = Vec::with_capacity(inputs.len());
        for (_, ty) in inputs {
            let limit = match ty {
                Type::Vector1 => 1,
                Type::Vector2 => 2,
                Type::Vector3 => 3,
                Type::Vector4 => 4,
            };
            defaults.push(default::InputDefault::new([0.0; 4], limit));
        }

        Self {
            width,
            title: Title::new(label),
            inputs_remote: vec![None; inputs.len()],
            outputs_remote: vec![None; outputs.len()],
            ports: Ports {
                inputs: inputs
                    .iter()
                    .map(|(name, ty)| port::State::new(name, *ty))
                    .collect(),
                outputs: outputs
                    .iter()
                    .map(|(name, ty)| port::State::new(name, *ty))
                    .collect(),
            },
            defaults,
        }
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

    pub fn update(&mut self, event: Event) {
        match event {
            Event::AttachInput(port, remote) => self.inputs_remote[port.0] = remote,
            Event::AttachOutput(port, remote) => self.outputs_remote[port.0] = remote,
            Event::SetDefault(port, value) => self.defaults[port.0].value = value,
            _ => (),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        node: NodeId,
        controls: impl Into<Option<Element<'a, Box<dyn DynMessage>, Renderer>>>,
    ) -> Element<'a, Message, Renderer> {
        let title = self.title.view(node);

        let title = Container::new(title).style(style::Node);
        let rule = Rule::horizontal(1).style(style::Transparent);

        let defaults =
            create_defaults_view(node, self.width, &mut self.defaults, &self.inputs_remote);

        let body = {
            let inputs = self.ports.input_view(node, &self.inputs_remote);
            let outputs = self.ports.output_view(node, &self.outputs_remote);

            let io = Row::new().push(inputs).push(outputs);

            let controls = match controls.into() {
                Some(controls) => controls.map(move |m| Message::Dynamic(node, m)),
                None => Space::new(Length::Shrink, Length::Shrink).into(),
            };

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
    pub inputs: Vec<port::State>,
    pub outputs: Vec<port::State>,
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
