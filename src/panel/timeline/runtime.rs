#[derive(Clone, Copy)]
pub enum Curve {
    Linear,
    Spline,
}

pub struct Animaton {
    pub name: String,
    pub bones: Vec<BoneFrame>,
}

pub struct Key<T> {
    pub time: u32,
    pub curve: Curve,
    pub data: T,
}

pub type LocationKey = Key<[f32; 2]>;
pub type RotationKey = Key<f32>;
pub type ScaleKey = Key<[f32; 2]>;

pub struct BoneFrame {
    pub label: String,
    pub open: bool,

    pub keys: Vec<u32>,
    pub location: Vec<LocationKey>,
    pub rotation: Vec<RotationKey>,
    pub scale: Vec<ScaleKey>,
}
