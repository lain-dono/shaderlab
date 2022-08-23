use super::Transform;
use egui::Vec2;
use std::ops::{Add, Mul};

#[derive(Clone, Copy)]
pub enum Interpolation<T> {
    Linear,
    Spline(T, T),
}

pub struct Animation {
    pub name: String,
    pub bones: Vec<BoneTimeline>,
}

impl Animation {
    pub fn duration(&self) -> u32 {
        self.bones
            .iter()
            .filter_map(|f| f.keys.last().copied())
            .max()
            .unwrap_or(0)
    }

    pub fn resolve_bone_tranform(&self, bone_index: usize, time: f32) -> Transform {
        self.bones
            .get(bone_index)
            .map(|clip| clip.resolve(time))
            .unwrap_or_default()
    }
}

pub struct Keyframe<T> {
    pub time: u32,
    pub curve: Interpolation<T>,
    pub value: T,
}

impl<T> Keyframe<T> {
    pub fn linear(time: u32, value: T) -> Self {
        Self {
            time,
            curve: Interpolation::Linear,
            value,
        }
    }
}

pub struct BoneTimeline {
    pub label: String,
    pub open: bool,

    pub keys: Vec<u32>,

    pub rotate: Timeline<f32>,
    pub translate: Timeline<Vec2>,
    pub scale: Timeline<Vec2>,
    pub shear: Timeline<Vec2>,
}

impl BoneTimeline {
    pub fn update_keys(&mut self) {
        self.keys.clear();

        self.keys.extend(self.rotate.time_iter());
        self.keys.extend(self.translate.time_iter());
        self.keys.extend(self.scale.time_iter());
        self.keys.extend(self.shear.time_iter());

        self.keys.dedup();
        self.keys.sort_unstable()
    }

    pub fn resolve(&self, time: f32) -> Transform {
        let rotate = self.rotate.resolve_or(time, 0.0);
        let translate = self.translate.resolve_or(time, Vec2::new(0.0, 0.0));
        let scale = self.scale.resolve_or(time, Vec2::new(1.0, 1.0));
        let shear = self.shear.resolve_or(time, Vec2::new(0.0, 0.0));
        let translate = translate.to_pos2();

        Transform {
            rotate,
            translate,
            scale,
            shear,
        }
    }
}

pub trait TimelineValue: Copy + Mul<f32, Output = Self> + Add<Self, Output = Self> {
    #[inline]
    fn linear(p0: Self, p1: Self, t: f32) -> Self {
        p0 * (1.0 - t) + p1 * t
    }

    #[inline]
    fn spline(p0: Self, p1: Self, p2: Self, p3: Self, t: f32) -> Self {
        let h = 1.0 - t;
        let p0 = p0 * (h * h * h);
        let p1 = p1 * (t * h * h * 3.0);
        let p2 = p2 * (t * t * h * 3.0);
        let p3 = p3 * (t * t * t);
        p0 + p1 + p2 + p3
    }
}

impl<T> TimelineValue for T where T: Copy + Mul<f32, Output = T> + Add<T, Output = T> {}

pub struct Timeline<T> {
    frames: Vec<Keyframe<T>>,
}

impl<T: TimelineValue> Timeline<T> {
    pub fn new(frames: Vec<Keyframe<T>>) -> Self {
        Self { frames }
    }

    pub fn get(&self, index: usize) -> (u32, Interpolation<T>, Option<u32>) {
        let curr = &self.frames[index];
        let next = self.frames.get(index + 1).map(|key| key.time);
        (curr.time, curr.curve, next)
    }

    pub fn add(&mut self, key: Keyframe<T>) {
        self.frames.push(key);
        self.frames.sort_by_key(|k| k.time);
    }

    pub fn remove(&mut self, time: u32) {
        self.frames.retain(|k| k.time != time);
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn find(&self, time: f32) -> Option<&Keyframe<T>> {
        self.find_pair(time).map(|[a, _]| a)
    }

    pub fn at(&self, time: u32) -> Option<&Keyframe<T>> {
        self.frames.iter().find(|a| a.time == time)
    }

    pub fn find_pair(&self, time: f32) -> Option<&[Keyframe<T>; 2]> {
        let frame = time.floor() as u32;

        crate::util::slice::array_windows(&self.frames)
            .find(|&[a, b]| a.time <= frame && frame < b.time)
    }

    pub fn resolve(&self, time: f32) -> Option<T> {
        self.find_pair(time).map(|[a, b]| {
            let (p0, p3) = (a.value, b.value);
            let (t0, t1) = (a.time as f32, b.time as f32);
            let t = (time - t0) / (t1 - t0);
            match a.curve {
                Interpolation::Linear => TimelineValue::linear(p0, p3, t),
                Interpolation::Spline(p1, p2) => TimelineValue::spline(p0, p1, p2, p3, t),
            }
        })
    }

    pub fn resolve_or(&self, time: f32, default: T) -> T {
        self.resolve(time)
            .or_else(|| self.frames.last().map(|a| a.value))
            .unwrap_or(default)
    }

    pub fn time_iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.frames.iter().map(|k| k.time)
    }
}
