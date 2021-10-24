use crate::node::{Message, NodeId, PortId};
use crate::style;
use iced_wgpu::Renderer;
use iced_winit::{alignment, text_input, Alignment, Container, Element, Length, Row};

pub struct InputDefault {
    pub value: [f64; 4],
    pub state: [text_input::State; 4],
    pub limit: usize,
}

impl InputDefault {
    pub fn new(value: [f64; 4], limit: usize) -> Self {
        assert!((1..=4).contains(&limit));
        let state = [
            text_input::State::default(),
            text_input::State::default(),
            text_input::State::default(),
            text_input::State::default(),
        ];
        Self {
            value,
            state,
            limit,
        }
    }

    pub fn view(&mut self, node: NodeId, port: PortId) -> Element<Message, Renderer> {
        let vector = self.value;
        let content: Element<Message, Renderer> = self
            .state
            .iter_mut()
            .enumerate()
            .take(self.limit)
            .fold(
                Row::new().align_items(Alignment::Start),
                |row, (n, state)| row.push(vector_float(state, vector, n, node, port)),
            )
            .into();
        Container::new(content).style(style::Node).padding(1).into()
    }
}

fn vector_float(
    state: &mut text_input::State,
    vector: [f64; 4],
    n: usize,
    node: NodeId,
    port: PortId,
) -> Element<Message, Renderer> {
    let label = ["x", "y", "z", "w"][n];
    let label = super::text_center(label)
        .vertical_alignment(alignment::Vertical::Top)
        .size(style::FONT_SIZE - 2)
        .width(Length::Units(14))
        .height(Length::Units(14));

    let value = format!("{:#?}", vector[n]);
    let input = style::Node::input(state, "", &value, move |value| {
        let mut vector = vector;
        vector[n] = value.parse::<f64>().unwrap_or(vector[n]);
        Message::SetDefault(node, port, vector)
    })
    .width(Length::Units(style::FONT_SIZE));

    Row::new().push(label).push(input).into()
}
