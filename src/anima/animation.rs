#[derive(Clone, Copy)]
pub enum Curve {
    Linear,
    Spline,
}

pub struct Animation {
    pub name: String,
    pub bones: Vec<BoneTimeline>,
}

pub struct Keyframe<T> {
    pub time: u32,
    pub curve: Curve,
    pub data: T,
}

pub struct BoneTimeline {
    pub label: String,
    pub open: bool,

    pub keys: Vec<u32>,

    pub rotation: Vec<Keyframe<f32>>,
    pub location: Vec<Keyframe<[f32; 2]>>,
    pub scale: Vec<Keyframe<[f32; 2]>>,
    pub shear: Vec<Keyframe<[f32; 2]>>,
}

impl BoneTimeline {
    pub fn update_keys(&mut self) {
        self.keys.clear();

        self.keys.extend(self.rotation.iter().map(|k| k.time));
        self.keys.extend(self.location.iter().map(|k| k.time));
        self.keys.extend(self.scale.iter().map(|k| k.time));
        self.keys.extend(self.shear.iter().map(|k| k.time));

        self.keys.dedup();
        self.keys.sort_unstable()
    }
}
