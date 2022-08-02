use super::{Animation, Armature, Bone, BoneTimeline, Interpolation, Keyframe};
use egui::*;

pub fn armature() -> Armature {
    Armature {
        slots: vec![],
        bones: vec![
            Bone {
                location: pos2(100.0, 30.0),
                length: 50.0,
                color: 0x99_00EECC,
                ..Default::default()
            },
            Bone {
                location: pos2(50.0, 0.0),
                length: 50.0,
                color: 0x99_EE00CC,
                parent: 0,
                ..Default::default()
            },
            Bone {
                location: pos2(50.0, 0.0),
                length: 50.0,
                color: 0x99_00EECC,
                parent: 1,
                ..Default::default()
            },
        ],
    }
}

pub fn animation() -> Animation {
    /*
    let bones = (0..40)
        .map(|i| BoneTimeline {
            label: format!("Bone #{}", i),
            open: false,
            keys: vec![],
            shear: vec![],
            location: (0..50)
                .filter_map(|j| {
                    ((j + i) % 3 == 0).then(|| Keyframe {
                        time: j as u32,
                        curve: if (j + i) % 2 == 0 {
                            Interpolation::Linear
                        } else {
                            Interpolation::Spline
                        },
                        value: vec2(0.0, 0.0),
                    })
                })
                .collect(),

            rotation: (0..50)
                .filter_map(|j| {
                    ((j + i) % 5 == 0).then(|| Keyframe {
                        time: j as u32,
                        curve: if (j + i) % 2 == 0 {
                            Interpolation::Linear
                        } else {
                            Interpolation::Spline
                        },
                        value: 0.0,
                    })
                })
                .collect(),

            scale: (0..50)
                .filter_map(|j| {
                    ((j + i) % 7 == 0).then(|| Keyframe {
                        time: j as u32,
                        curve: if (j + i) % 2 == 0 {
                            Interpolation::Linear
                        } else {
                            Interpolation::Spline
                        },
                        value: vec2(1.0, 1.0),
                    })
                })
                .collect(),
        })
        .collect();
    */

    let bones = vec![BoneTimeline {
        label: String::from("bone0"),
        open: false,
        keys: vec![],
        location: vec![],
        rotation: vec![
            Keyframe {
                time: 0,
                curve: Interpolation::Linear,
                value: 0.0,
            },
            Keyframe {
                time: 5,
                curve: Interpolation::Linear,
                value: std::f32::consts::FRAC_PI_2,
            },
            Keyframe {
                time: 8,
                curve: Interpolation::Linear,
                value: std::f32::consts::PI,
            },
        ],
        scale: vec![],
        shear: vec![],
    }];

    let name = String::from("some animation");

    Animation { name, bones }
}
