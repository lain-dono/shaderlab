use crate::workspace::{
    Data, Direction, Input, InputDefault, Output, Port, PortData, Preview, PreviewBuilder, Stage,
    Storage,
};
use egui::style::Margin;
use egui::*;
use slotmap::SlotMap;

slotmap::new_key_type! {
    pub struct Node;
}

pub struct NodeBuilder<'a> {
    pub storage: &'a mut Storage,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

impl<'a> NodeBuilder<'a> {
    pub fn input(
        &mut self,
        node: Node,
        label: impl Into<String>,
        stage: Stage,
        data: Data,
        default: impl Into<Option<InputDefault>>,
    ) -> Port {
        self.port(node, Input, stage, label.into(), data, default.into())
    }

    pub fn output(
        &mut self,
        node: Node,
        label: impl Into<String>,
        stage: Stage,
        data: Data,
        default: impl Into<Option<InputDefault>>,
    ) -> Port {
        self.port(node, Output, stage, label.into(), data, default.into())
    }

    fn port(
        &mut self,
        node: Node,
        direction: Direction,
        stage: Stage,
        label: String,
        data: Data,
        default: Option<InputDefault>,
    ) -> Port {
        let input_default = default
            .or_else(|| match data {
                Data::Boolean => Some(InputDefault::bool()),
                Data::Float => Some(InputDefault::float(0.0)),
                Data::Vector2 => Some(InputDefault::vector2(0.0, 0.0)),
                Data::Vector3 => Some(InputDefault::vector3(0.0, 0.0, 0.0)),
                Data::Vector4 => Some(InputDefault::vector4(0.0, 0.0, 0.0, 1.0)),

                Data::VectorAny => Some(InputDefault::vector4(0.0, 0.0, 0.0, 1.0)),
                Data::FloatOrVector => Some(InputDefault::float(0.0)),

                _ => None,
            })
            .filter(|_| direction.is_input());

        let key = self.storage.ports.insert(PortData {
            node,
            direction,
            stage,
            label,
            data,

            position: Pos2::ZERO,
            rect: Rect::from_min_max(Pos2::ZERO, Pos2::ZERO),

            links: Default::default(),
            input_default,
        });

        match direction {
            Direction::Input => self.inputs.push(key),
            Direction::Output => self.outputs.push(key),
        }

        key
    }
}

pub enum NodeInteraction {
    Remove,
    Drag(Vec2),
    PortHovered(Port),
    LinkStart(Port),
    LinkEnd(Port),
}

pub struct NodeData {
    pub title: String,
    pub width: f32,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,

    pub preview_is_valid: bool,
    pub show_preview: bool,
    pub position: Pos2,
    pub rect: Rect,
    pub layer: Option<LayerId>,

    pub preview: Option<Preview>,
    pub builder: Box<dyn PreviewBuilder>,
}

impl NodeData {
    pub fn new(
        title: impl Into<String>,
        width: f32,
        builder: impl PreviewBuilder + 'static,
    ) -> Self {
        Self {
            title: title.into(),
            width,
            inputs: Vec::new(),
            outputs: Vec::new(),

            preview_is_valid: false,
            show_preview: false,
            position: Pos2::new(f32::INFINITY, f32::INFINITY),
            rect: Rect::NOTHING,
            layer: None,

            preview: None,
            builder: Box::new(builder),
        }
    }

    pub fn draw(
        &mut self,
        ctx: &Context,
        ports: &mut SlotMap<Port, PortData>,
        key: Node,
        offset: Vec2,
        selected: Option<Color32>,
        interactions: &mut Vec<NodeInteraction>,
    ) {
        let mut area = Area::new((key, "node"));

        if self.position.is_finite() {
            area = area.current_pos(self.position + offset);
        }

        self.layer = Some(area.layer());

        let InnerResponse { response, .. } = area.show(ctx, |ui| {
            let mut frame = Frame::window(ui.style());

            frame.shadow.extrusion = 0.0;
            frame.stroke.color = selected.unwrap_or(frame.stroke.color);
            frame.rounding = Rounding::none();
            frame.margin = Margin::same(0.0);
            frame.fill = frame.fill.linear_multiply(0.95);

            frame.show(ui, |ui| {
                ui.set_width(self.width);
                self.draw_body(ui, ports, interactions)
            });
        });

        let drag_delta = if response.dragged_by(PointerButton::Primary) {
            let drag_delta = response.drag_delta();
            interactions.push(NodeInteraction::Drag(drag_delta));
            Some(drag_delta)
        } else {
            None
        };

        if self.position.is_finite() {
            for &key in &self.inputs {
                let port = &mut ports[key];
                if !port.links.is_empty() {
                    continue;
                }

                if let Some(default) = port.input_default.as_mut() {
                    let width = default.width.unwrap_or(1000.0);
                    let pos = port.rect.left_top() - egui::vec2(width + 10.0, 0.0);

                    let area = Area::new((key, "_InputDefault")).movable(false);
                    if drag_delta.is_some() {
                        ctx.move_to_top(area.layer())
                    }
                    default.layer_id = Some(area.layer());
                    let response = area.current_pos(pos).show(ctx, |ui| {
                        let frame = Frame {
                            fill: Frame::window(ui.style()).fill.linear_multiply(0.95),
                            margin: Margin::symmetric(4.0, 1.0),
                            ..Frame::none()
                        };
                        let layout = Layout::top_down(Align::Max);

                        ui.set_width(width);
                        ui.with_layout(layout, |ui| {
                            frame.show(ui, |ui| ui.horizontal(|ui| default.ui(ui)).response)
                        })
                        .inner
                    });

                    default.width = Some(response.inner.inner.rect.width());
                }
            }
        }

        self.position = response.rect.min - offset;
        self.rect = response.rect;
    }

    pub fn draw_body(
        &mut self,
        ui: &mut Ui,
        ports: &mut SlotMap<Port, PortData>,
        interactions: &mut Vec<NodeInteraction>,
    ) {
        let margin = Margin::symmetric(0.0, 4.0);
        Frame::none().margin(margin).show(ui, |ui| {
            ui.horizontal(|ui| {
                let mark = if self.show_preview { "⏷" } else { "⏵" };
                if ui.add(Button::new(mark).frame(false)).clicked() {
                    self.show_preview = !self.show_preview;
                }
                ui.centered_and_justified(|ui| ui.label(&self.title));
                if ui.add(Button::new("❌").frame(false)).clicked() {
                    interactions.push(NodeInteraction::Remove);
                }
            });
        });

        fn run_response(interactions: &mut Vec<NodeInteraction>, port: Port, response: Response) {
            if response.hovered() {
                interactions.push(NodeInteraction::PortHovered(port));
            }
            if response.drag_started() {
                interactions.push(NodeInteraction::LinkStart(port));
            }
            if response.drag_released() {
                interactions.push(NodeInteraction::LinkEnd(port));
            }
        }

        let margin = Margin::symmetric(2.0, 0.0);
        Frame::none().margin(margin).show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            let mut inputs = self.inputs.iter();
            let mut outputs = self.outputs.iter();

            for _ in 0..usize::min(self.inputs.len(), self.outputs.len()) {
                ui.columns(2, |ui| {
                    let port = *inputs.next().unwrap();
                    run_response(interactions, port, ports[port].widget(&mut ui[0]));

                    let port = *outputs.next().unwrap();
                    run_response(interactions, port, ports[port].widget(&mut ui[1]));
                });
            }
            for &port in inputs {
                run_response(interactions, port, ports[port].widget(ui));
            }
            for &port in outputs {
                run_response(interactions, port, ports[port].widget(ui));
            }
        });

        let margin = Margin::symmetric(4.0, 4.0);
        Frame::none().margin(margin).show(ui, |ui| {
            self.builder.ui(ui);
        });

        if self.show_preview && self.preview_is_valid && self.builder.show_preview() {
            let margin = Margin::symmetric(1.0, 1.0);
            Frame::none().margin(margin).show(ui, |ui| {
                if let Some(preview) = self.preview.as_ref() {
                    ui.image(preview.texture_id, preview.size);
                }
            });
        }
    }
}
