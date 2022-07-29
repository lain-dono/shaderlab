use super::Matrix;
use egui::*;

pub struct Slot {
    pub name: String,
    pub bone: usize,
}

bitflags::bitflags! {
    pub struct BoneFlags: u32 {

    }
}

pub struct Bone {
    pub name: String,
    pub rotation: f32,
    pub location: Pos2,
    pub scale: Vec2,
    pub skew: Vec2,

    pub color: u32,
    pub length: f32,
    pub parent: u32,
    pub flags: BoneFlags,
}

impl Default for Bone {
    fn default() -> Self {
        Self {
            name: String::new(),
            rotation: 0.0,
            location: pos2(0.0, 0.0),
            scale: vec2(1.0, 1.0),
            skew: vec2(0.0, 0.0),

            color: 0xFFFFFFFF,
            length: 0.0,
            parent: u32::max_value(),

            flags: BoneFlags::empty(),
        }
    }
}

impl Bone {
    pub fn local_matrix(&self) -> Matrix {
        let (sx, cx) = (self.rotation - self.skew.x).sin_cos();
        let (sy, cy) = (self.rotation + self.skew.y).sin_cos();
        let sx = -sx;

        Matrix {
            a: cy * self.scale.x,
            b: sy * self.scale.x,
            c: sx * self.scale.y,
            d: cx * self.scale.y,
            tx: self.location.x,
            ty: self.location.y,
        }
    }
}

pub struct Armature {
    pub bones: Vec<Bone>,
    pub slots: Vec<Slot>,
}

impl Armature {
    pub fn insert_bone(&mut self, index: usize, bone: Bone) {
        for bone in &mut self.bones {
            if bone.parent >= index as u32 {
                bone.parent = bone.parent.saturating_add(1);
            }
        }
        self.bones.insert(index, bone);
    }

    pub fn add_bone(&mut self, bone: Bone) {
        self.bones.push(bone);
    }

    pub fn paint_bones(&self, painter: &Painter, offset: Pos2, zoom: f32, world: &[Matrix]) {
        let screen = Matrix {
            a: zoom,
            b: 0.0,
            c: 0.0,
            d: -zoom,
            tx: offset.x,
            ty: offset.y,
        };

        for (bone, &world) in self.bones.iter().zip(world) {
            let transform = screen.prepend(world);
            let [b, g, r, a] = bone.color.to_le_bytes();
            let color = Color32::from_rgba_unmultiplied(r, g, b, a);
            let length = bone.length;

            let extra = vec2(length * 0.2, length * 0.1);
            let outer_radius = (zoom * length * 0.075).max(2.0);
            let inner_radius = outer_radius * 0.8;

            let a = transform.apply(0.0, 0.0).into();
            let b = transform.apply(extra.x, -extra.y).into();
            let c = transform.apply(length, 0.0).into();
            let d = transform.apply(extra.x, extra.y).into();

            let points = vec![a, b, c, d];

            let stroke = Stroke::default();
            painter.add(Shape::Path(epaint::PathShape::convex_polygon(
                points, color, stroke,
            )));
            painter.circle_filled(a, outer_radius, color);
            painter.circle_filled(a, inner_radius, Color32::BLACK);
        }
    }
}
