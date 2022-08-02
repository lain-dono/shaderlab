use super::Matrix;
use egui::Vec2;

#[derive(Clone, Copy)]
pub enum Interpolation {
    Linear,
    Spline,
}

pub struct Animation {
    pub name: String,
    pub bones: Vec<BoneTimeline>,
}

pub struct Keyframe<T> {
    pub time: u32,
    pub curve: Interpolation,
    pub value: T,
}

pub struct BoneTimeline {
    pub label: String,
    pub open: bool,

    pub keys: Vec<u32>,

    pub rotation: Vec<Keyframe<f32>>,
    pub location: Vec<Keyframe<Vec2>>,
    pub scale: Vec<Keyframe<Vec2>>,
    pub shear: Vec<Keyframe<Vec2>>,
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

    pub fn resolve(&self, time: f32) -> Matrix {
        let rotation = resolve(&self.rotation, time).unwrap_or(0.0);
        let location = resolve(&self.location, time).unwrap_or(Vec2::new(0.0, 0.0));
        let scale = resolve(&self.scale, time).unwrap_or(Vec2::new(1.0, 1.0));
        let shear = resolve(&self.shear, time).unwrap_or(Vec2::new(0.0, 0.0));

        let location = location.to_pos2();

        super::transform(rotation, location, scale, shear)
    }
}

use std::ops::{Add, Mul};

fn resolve<T>(frames: &[Keyframe<T>], time: f32) -> Option<T>
where
    T: Copy + Mul<f32, Output = T> + Add<T, Output = T>,
{
    let frame = time.floor() as u32;

    crate::util::slice::array_windows(frames)
        .find(|&[a, b]| a.time <= frame && frame < b.time)
        .map(|[a, b]| {
            let (p0, p1) = (a.value, b.value);
            let (t0, t1) = (a.time as f32, b.time as f32);
            let t = (time - t0) / (t1 - t0);
            linear(p0, p1, t)
        })
        .or_else(|| frames.last().map(|a| a.value))
}

#[inline]
fn linear<T>(p0: T, p1: T, t: f32) -> T
where
    T: Mul<f32, Output = T> + Add<T, Output = T>,
{
    p0 * (1.0 - t) + p1 * t
}

#[inline]
fn bezier<T>(p0: T, p1: T, p2: T, p3: T, t: f32) -> T
where
    T: Mul<f32, Output = T> + Add<T, Output = T>,
{
    let h = 1.0 - t;
    let p0 = p0 * (h * h * h);
    let p1 = p1 * (t * h * h * 3.0);
    let p2 = p2 * (t * t * h * 3.0);
    let p3 = p3 * (t * t * t);
    p0 + p1 + p2 + p3
}
