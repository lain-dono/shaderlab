use crate::workspace::{Link, Node};
use ahash::AHashSet;
use egui::text::LayoutJob;
use egui::*;

slotmap::new_key_type! {
    pub struct Port;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Data {
    Boolean,

    Float,
    Vector2,
    Vector3,
    Vector4,
    VectorAny,

    Matrix2,
    Matrix3,
    Matrix4,
    MatrixAny,

    VectorOrMatrix,
    FloatOrVector,
    FloatOrVectorOrMatrix,

    Image(naga::ImageClass),
    VirtualTexture,
    Gradient,
    Sampler,
}

impl Data {
    pub fn can_connect(self, other: Self) -> bool {
        use Data::*;
        match (self, other) {
            (Image(a), Image(b)) => a == b,
            (VirtualTexture, VirtualTexture) => true,
            (Gradient, Gradient) => true,
            (Sampler, Sampler) => true,
            (Boolean, Boolean)
            | (
                Float | FloatOrVector | FloatOrVectorOrMatrix,
                Float | FloatOrVector | FloatOrVectorOrMatrix,
            )
            | (
                Vector2 | Vector3 | Vector4 | VectorAny | FloatOrVector | FloatOrVectorOrMatrix,
                Vector2 | Vector3 | Vector4 | VectorAny | FloatOrVector | FloatOrVectorOrMatrix,
            )
            | (
                Matrix2 | Matrix3 | Matrix4 | MatrixAny | FloatOrVectorOrMatrix,
                Matrix2 | Matrix3 | Matrix4 | MatrixAny | FloatOrVectorOrMatrix,
            ) => true,

            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Input,
    Output,
}

impl Direction {
    pub fn is_input(self) -> bool {
        matches!(self, Self::Input)
    }

    pub fn is_output(self) -> bool {
        matches!(self, Self::Output)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Vertex,
    Fragment,
}

impl Stage {
    pub fn is_fragment(self) -> bool {
        matches!(self, Self::Fragment)
    }

    pub fn is_vertex(self) -> bool {
        matches!(self, Self::Vertex)
    }

    pub fn draw(self, painter: &Painter, center: Pos2, fill_color: Color32, linked: bool) {
        let stroke = (1.5, fill_color);
        let radius = 4.0;
        let rect = Rect::from_center_size(center, [7.0; 2].into());

        match self {
            Self::Fragment => {
                painter.add(Shape::circle_stroke(center, radius, stroke));
                if linked {
                    painter.add(Shape::circle_filled(center, radius - 2.0, fill_color));
                }
            }
            Self::Vertex => {
                painter.add(Shape::rect_stroke(rect, 0.0, stroke));
                if linked {
                    painter.add(Shape::rect_filled(rect.shrink(2.0), 0.0, fill_color));
                }
            }
        }
    }
}

pub struct PortData {
    pub label: String,
    pub direction: Direction,
    pub stage: Stage,
    pub data: Data,
    pub node: Node,

    pub position: Pos2,
    pub rect: Rect,
    pub links: AHashSet<Link>,

    pub input_default: Option<InputDefault>,
}

impl PortData {
    pub fn input(node: Node, label: impl Into<String>, stage: Stage, data: Data) -> Self {
        Self {
            label: label.into(),
            direction: Direction::Input,
            stage,
            data,
            node,

            position: Pos2::ZERO,
            rect: Rect::from_min_max(Pos2::ZERO, Pos2::ZERO),
            links: AHashSet::default(),

            input_default: None,
        }
    }

    pub fn output(node: Node, label: impl Into<String>, stage: Stage, data: Data) -> Self {
        Self {
            label: label.into(),
            direction: Direction::Output,
            stage,
            data,
            node,

            position: Pos2::ZERO,
            rect: Rect::from_min_max(Pos2::ZERO, Pos2::ZERO),
            links: AHashSet::default(),

            input_default: None,
        }
    }

    pub fn is_input(&self) -> bool {
        matches!(self.direction, Direction::Input)
    }

    pub fn is_output(&self) -> bool {
        matches!(self.direction, Direction::Output)
    }

    pub fn widget(&mut self, ui: &mut Ui) -> Response {
        let output = self.is_output();
        let linked = !self.links.is_empty();

        let align = if output { Align::Max } else { Align::Min };
        let layout = Layout::top_down_justified(align);

        let InnerResponse { inner, response } = ui.with_layout(layout, |ui| {
            let sense = Sense::drag();

            let button_padding = ui.spacing().button_padding;
            let total_extra = button_padding + button_padding;
            let wrap_width = ui.available_width() - total_extra.x;

            let text = WidgetText::from({
                let text = &self.label;
                let font_id = FontId::proportional(14.0);
                let mut job = LayoutJob::simple_singleline(text.into(), font_id, Color32::WHITE);
                job.halign = align;
                job
            });

            let mut text = text.into_galley(ui, None, wrap_width, TextStyle::Button);
            text.galley_has_color = false;

            let icon_width = ui.spacing().icon_width;
            let icon_spacing = ui.spacing().icon_spacing;

            let mut desired_size = text.size() + total_extra;
            desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);

            desired_size.x += icon_width + icon_spacing;
            desired_size.y = desired_size.y.max(icon_width + total_extra.y);

            let (rect, response) = ui.allocate_at_least(desired_size, sense);
            // ? response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, text.text()));

            let ext_hover = {
                let pos = ui.ctx().input().pointer.hover_pos();
                pos.filter(|&pos| response.rect.contains(pos))
                    .and_then(|pos| ui.ctx().layer_id_at(pos))
                    .map_or(false, |layer| layer == ui.painter().layer_id())
            };

            if ui.is_rect_visible(rect) {
                let Rect { min, max } = rect;
                let visuals = ui.style().interact(&response);

                let pad = button_padding.x + icon_width + icon_spacing;
                let x = if output { max.x - pad } else { min.x + pad };
                let text_pos = pos2(x, rect.center().y - 0.5 * text.size().y);
                text.paint_with_visuals(ui.painter(), text_pos, visuals);

                let pad = icon_width * 0.5;
                let x = if output { max.x - pad } else { min.x + pad };
                let center = pos2(x, rect.center().y);

                let linked = linked || response.hovered() || response.dragged() || ext_hover;
                let fill_color = self.data.color();
                self.stage.draw(ui.painter(), center, fill_color, linked);

                self.position = center;
            }

            response
        });

        let response = response.union(inner);
        self.rect = response.rect;
        response
    }
}

pub enum InputDefaultType {
    Marker(String),
    Bool,
    Float,
    Vector2,
    Vector3,
    Vector4,
}

pub struct InputDefault {
    pub kind: InputDefaultType,
    pub width: Option<f32>,
    pub layer_id: Option<LayerId>,

    pub checked: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl InputDefault {
    pub const fn new(kind: InputDefaultType, x: f64, y: f64, z: f64, w: f64) -> Self {
        Self {
            kind,
            width: None,
            layer_id: None,
            checked: false,
            x,
            y,
            z,
            w,
        }
    }

    pub fn marker(marker: impl Into<String>) -> Self {
        Self::new(InputDefaultType::Marker(marker.into()), 0.0, 0.0, 0.0, 1.0)
    }

    pub const fn bool() -> Self {
        Self::new(InputDefaultType::Bool, 0.0, 0.0, 0.0, 1.0)
    }

    pub const fn float(x: f64) -> Self {
        Self::new(InputDefaultType::Float, x, 0.0, 0.0, 1.0)
    }

    pub const fn vector2(x: f64, y: f64) -> Self {
        Self::new(InputDefaultType::Vector2, x, y, 0.0, 1.0)
    }

    pub const fn vector3(x: f64, y: f64, z: f64) -> Self {
        Self::new(InputDefaultType::Vector3, x, y, z, 1.0)
    }

    pub const fn vector4(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self::new(InputDefaultType::Vector4, x, y, z, w)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        fn drag_value(ui: &mut egui::Ui, label: &str, value: &mut f64) -> egui::Response {
            let widget = egui::DragValue::new(value);
            ui.add(widget.speed(0.01).prefix(label))
        }

        match self.kind {
            InputDefaultType::Marker(ref marker) => ui.label(marker),
            InputDefaultType::Bool => ui.checkbox(&mut self.checked, ""),
            InputDefaultType::Float => drag_value(ui, "x ", &mut self.x),
            InputDefaultType::Vector2 => {
                let x = drag_value(ui, "y ", &mut self.y);
                let y = drag_value(ui, "x ", &mut self.x);
                x.union(y)
            }
            InputDefaultType::Vector3 => {
                let x = drag_value(ui, "z ", &mut self.z);
                let y = drag_value(ui, "y ", &mut self.y);
                let z = drag_value(ui, "x ", &mut self.x);
                x.union(y).union(z)
            }
            InputDefaultType::Vector4 => {
                let x = drag_value(ui, "w ", &mut self.w);
                let y = drag_value(ui, "z ", &mut self.z);
                let z = drag_value(ui, "y ", &mut self.y);
                let w = drag_value(ui, "x ", &mut self.x);
                x.union(y).union(z).union(w)
            }
        }
    }
}

impl crate::builder::expr::Emit for InputDefault {
    fn emit(&self, function: &mut crate::builder::FnBuilder) -> crate::builder::EmitResult {
        use crate::builder::expr::{Bool, Float};
        let [x, y, z, w] = [Float(self.x), Float(self.y), Float(self.z), Float(self.w)];
        match self.kind {
            InputDefaultType::Bool => Bool(self.checked).emit(function),
            InputDefaultType::Float => x.emit(function),
            InputDefaultType::Vector2 => [x, y].emit(function),
            InputDefaultType::Vector3 => [x, y, z].emit(function),
            InputDefaultType::Vector4 => [x, y, z, w].emit(function),
            _ => unimplemented!(),
        }
    }
}
