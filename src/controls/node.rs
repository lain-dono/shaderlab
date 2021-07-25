use super::{
    edge::{Pending, PortId},
    port::Port,
    Message,
};
use crate::style::{self, FONT_SIZE};
use crate::widget::pad;
use iced_wgpu::Renderer;
use iced_winit::{
    Align, Column, Container, Element, HorizontalAlignment, Length, Point, Row, Rule, Space, Text,
    VerticalAlignment,
};

slotmap::new_key_type! { pub struct NodeId; }

impl ToString for NodeId {
    fn to_string(&self) -> String {
        let value = slotmap::Key::data(self).as_ffi();
        let idx = (value & 0xffff_ffff) as u32;
        let version = ((value >> 32) | 1) as u32; // Ensure version is odd.
        format!("{}v{}", idx, version)
    }
}

pub struct NodeWidget {
    pub id: NodeId,
    pub position: Point,

    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,

    pub node: Box<dyn crate::node::Node>,

    pub title_state: pad::State,
    pub close: pad::State,
    pub drag: pad::State,
}

impl NodeWidget {
    pub fn new(id: NodeId, position: Point, node: Box<dyn crate::node::Node>) -> Self {
        Self {
            id,
            position,
            inputs: node.inputs().iter().map(Port::new).collect(),
            outputs: node.outputs().iter().map(Port::new).collect(),
            node,
            title_state: Default::default(),
            close: Default::default(),
            drag: Default::default(),
        }
    }
    pub fn label(&self) -> &str {
        self.node.label()
    }

    pub fn widget(&mut self) -> Element<Message, Renderer> {
        let node = self.id;

        fn text_center(label: &str) -> Text<Renderer> {
            Text::new(label)
                .size(FONT_SIZE)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center)
        }

        fn grap_pad<'a>(
            node: NodeId,
            state: &'a mut pad::State,
            content: impl Into<Element<'a, Message, Renderer>>,
        ) -> pad::Pad<Message> {
            pad::Pad::new(state, content)
                .padding([2, 0])
                .on_press(Message::StartDrag(node))
                .on_release(Message::EndDrag(node))
                .interaction(iced_native::mouse::Interaction::Grab)
        }

        fn create_ports(
            node: NodeId,
            ports: &mut [Port],
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
            let drag = text_center("=").size(16).width(Length::Units(FONT_SIZE));
            let drag = grap_pad(self.id, &mut self.drag, drag);

            let title = text_center(self.node.label()).width(Length::Fill);
            let title = grap_pad(self.id, &mut self.title_state, title).width(Length::Fill);

            let close = text_center("×").size(16).width(Length::Units(FONT_SIZE));
            let close = pad::Pad::new(&mut self.close, close)
                .padding([2, 0])
                .on_release(Message::RemoveNode(self.id));

            Row::new()
                .width(Length::Fill)
                .align_items(Align::Center)
                .push(drag)
                .push(title)
                .push(close)
        };

        let inputs = create_ports(node, &mut self.inputs, Pending::input);
        let outputs = create_ports(node, &mut self.outputs, Pending::output);

        let rule = Rule::horizontal(0).style(style::Node);
        let io = Row::new().push(inputs).push(outputs);

        let width = Length::Units(self.node.width());
        let inner = Column::new().push(title).push(rule).push(io).push(
            self.node
                .view(node)
                .map(move |m| Message::NodeInternal(node, m)),
        );

        Container::new(inner)
            .style(style::Node)
            .width(width)
            .height(Length::Shrink)
            .into()
    }
}
