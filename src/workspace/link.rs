use crate::workspace::{Data, Node, Port};

slotmap::new_key_type! {
    pub struct Link;
}

#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub node: Node,
    pub port: Port,
}

impl Slot {
    pub const fn new(node: Node, port: Port) -> Self {
        Self { node, port }
    }
}

pub struct LinkData {
    pub min: Slot, // output
    pub max: Slot, // input

    pub shape: Option<egui::layers::ShapeIdx>,
}

impl LinkData {
    pub fn has(&self, port: Port) -> bool {
        self.min.port == port || self.max.port == port
    }

    pub fn output_for(&self, input: Port) -> Option<Port> {
        self.has_input(input).then(|| self.min.port)
    }
    pub fn has_output(&self, port: Port) -> bool {
        self.min.port == port
    }

    pub fn has_input(&self, port: Port) -> bool {
        self.max.port == port
    }
}

#[derive(Debug)]
pub struct LinkBezier(pub [egui::Pos2; 4]);

impl LinkBezier {
    #[inline]
    pub fn new(min: egui::Pos2, max: egui::Pos2) -> Self {
        let x = (min.distance(max) / 2.0).min(150.0);
        Self([min, min + egui::vec2(x, 0.0), max - egui::vec2(x, 0.0), max])
    }

    pub fn validate(self) -> Option<Self> {
        let Self([a, b, c, d]) = self;

        if !a.is_finite() || !b.is_finite() || !c.is_finite() || !d.is_finite() {
            return None;
        }

        if a.distance(d) > 2.0 {
            Some(Self([a, b, c, d]))
        } else {
            None
        }
    }

    #[inline]
    pub fn eval(&self, t: f32) -> egui::Pos2 {
        let (a, b, c, d) = (
            self.0[0].to_vec2(),
            self.0[1].to_vec2(),
            self.0[2].to_vec2(),
            self.0[3].to_vec2(),
        );

        let pt = (1.0 - t).powi(3) * a
            + 3.0 * (1.0 - t).powi(2) * t * b
            + 3.0 * (1.0 - t) * t.powi(2) * c
            + t.powi(3) * d;

        <[f32; 2]>::from(pt).into()
    }

    #[inline]
    pub fn containing_rect_for_bezier_curve(&self, hover_distance: f32) -> egui::Rect {
        let min = self.0[0].min(self.0[3]);
        let max = self.0[0].max(self.0[3]);

        let mut rect = egui::Rect::from_min_max(min, max);
        rect.extend_with(self.0[1]);
        rect.extend_with(self.0[2]);
        rect.expand(hover_distance)
    }

    fn to_egui(&self, stroke: impl Into<egui::Stroke>) -> egui::epaint::CubicBezierShape {
        let fill = egui::Color32::TRANSPARENT;
        egui::epaint::CubicBezierShape::from_points_stroke(self.0, false, fill, stroke)
    }

    pub fn draw(&self, stroke: impl Into<egui::Stroke>) -> egui::Shape {
        egui::Shape::CubicBezier(self.to_egui(stroke))
    }

    #[inline]
    pub fn distance_to_cubic_bezier(&self, pos: egui::Pos2) -> f32 {
        let segments = 0.1;
        let link_length = self.0[0].distance(self.0[3]);
        let num_segments = 1.max((link_length * segments) as usize);

        let mut last = self.0[0];
        let mut closest = self.0[0];
        let mut distance = f32::MAX;
        let t_step = 1.0 / num_segments as f32;
        for i in 1..num_segments {
            let current = self.eval(t_step * i as f32);
            let line = line_closest_point(last, current, pos);
            let dist = pos.distance_sq(line);
            if dist < distance {
                closest = line;
                distance = dist;
            }
            last = current;
        }

        pos.distance(closest)
    }
}

#[inline]
fn line_closest_point(a: egui::Pos2, b: egui::Pos2, p: egui::Pos2) -> egui::Pos2 {
    let ap = p - a;
    let ab_dir = b - a;
    let dot = ap.x * ab_dir.x + ap.y * ab_dir.y;
    let ab_len_sqr = ab_dir.x * ab_dir.x + ab_dir.y * ab_dir.y;
    if dot < 0.0 {
        a
    } else if dot <= ab_len_sqr {
        a + ab_dir * dot / ab_len_sqr
    } else {
        b
    }
}

impl Data {
    pub fn color(self) -> egui::Color32 {
        use Data::*;
        match self {
            Boolean => PORT_BOOL,
            Float | FloatOrVector | VectorAny => PORT_VECTOR_1,
            Vector2 => PORT_VECTOR_2,
            Vector3 => PORT_VECTOR_3,
            Vector4 => PORT_VECTOR_4,
            FloatOrVectorOrMatrix | VectorOrMatrix => PORT_MATRIX,
            Matrix2 | Matrix3 | Matrix4 | MatrixAny => PORT_MATRIX,
            Image(_) => PORT_IMAGE,
            VirtualTexture | Gradient | Sampler => PORT_STRUCT,
        }
    }
}

use egui::Color32;

pub const PORT_BOOL: Color32 = Color32::from_rgb(0x94, 0x81, 0xe6);
pub const PORT_STRUCT: Color32 = Color32::from_rgb(0xc8, 0xc8, 0xc8);
pub const PORT_IMAGE: Color32 = Color32::from_rgb(0xff, 0x8b, 0x8b);
pub const PORT_MATRIX: Color32 = Color32::from_rgb(0x8f, 0xc1, 0xdf);

pub const PORT_VECTOR_1: Color32 = Color32::from_rgb(0x84, 0xe4, 0xe7);
pub const PORT_VECTOR_2: Color32 = Color32::from_rgb(0x9a, 0xef, 0x92);
pub const PORT_VECTOR_3: Color32 = Color32::from_rgb(0xf6, 0xff, 0x9a);
pub const PORT_VECTOR_4: Color32 = Color32::from_rgb(0xfb, 0xcb, 0xf4);

/*
pub const WORKSPACE_BG: Color32 = Color32::from_rgb(0x20, 0x20, 0x20);
//pub const WORKSPACE_GRID: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1C);
pub const WORKSPACE_GRID: Color32 = Color32::from_rgb(0x20, 0x20, 0x20);

//pub const NODE_BACKGROUND: Color32 = rgba(0x333333, 0.95);
pub const NODE_TEXT: Color32 = Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub const NODE_BASE: Color32 = Color32::from_rgba_premultiplied(0x33, 0x33, 0x33, 0xEE); // 0xE6
pub const NODE_ACTIVE: Color32 = Color32::from_rgba_premultiplied(0x44, 0x44, 0x44, 0xEE);
pub const NODE_BORDER: Color32 = Color32::from_rgba_premultiplied(0x18, 0x18, 0x18, 0xEE);
pub const NODE_BORDER_THICKNESS: f32 = 2.0;
pub const NODE_SEPARATOR_THICKNESS: f32 = 1.0;

pub const TITLE_PADDING: egui::Vec2 = egui::vec2(8.0, 4.0);
pub const NODE_PADDING: egui::Vec2 = egui::vec2(16.0, 4.0);

pub const SLIDER_RAIL: Color32 = Color32::from_rgb(0x5e, 0x5e, 0x5e);
pub const SLIDER_HANDLE_ACTIVE: Color32 = Color32::from_rgb(0x99, 0x99, 0x99);
pub const SLIDER_HANDLE_HOVER: Color32 = Color32::from_rgb(0xEA, 0xEA, 0xEA);

pub const FONT_SIZE: u16 = 14;

pub const PORT_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(0x33, 0x33, 0x33, 0xe6);

pub const SELECTION: Color32 = Color32::from_rgb(0x44, 0xc0, 0xff);

pub const SIDEBAR_BG: Color32 = Color32::from_rgb(39, 39, 39);

*/
