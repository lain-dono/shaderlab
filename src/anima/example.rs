use super::{
    runtime::Transform, Animation, Armature, Bone, BoneTimeline, Interpolation, Keyframe, Timeline,
};
use egui::*;

pub fn armature() -> Armature {
    Armature {
        slots: vec![],
        bones: vec![
            Bone {
                transform: Transform {
                    translate: pos2(100.0, 30.0),
                    ..Default::default()
                },
                length: 50.0,
                color: 0x99_00EECC,
                ..Default::default()
            },
            Bone {
                transform: Transform {
                    translate: pos2(50.0, 0.0),
                    ..Default::default()
                },
                length: 50.0,
                color: 0x99_EE00CC,
                parent: 0,
                ..Default::default()
            },
            Bone {
                transform: Transform {
                    translate: pos2(50.0, 0.0),
                    ..Default::default()
                },
                length: 50.0,
                color: 0x99_00EECC,
                parent: 1,
                ..Default::default()
            },
        ],
    }
}

pub fn animation() -> Animation {
    let bones = vec![
        BoneTimeline {
            label: String::from("bone0"),
            open: false,
            keys: vec![],
            translate: Timeline::new(vec![]),
            rotate: Timeline::new(vec![
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
            ]),
            scale: Timeline::new(vec![]),
            shear: Timeline::new(vec![]),
        },
        BoneTimeline {
            label: String::from("bone1"),
            open: false,
            keys: vec![],
            translate: Timeline::new(vec![]),
            rotate: Timeline::new(vec![]),
            scale: Timeline::new(vec![]),
            shear: Timeline::new(vec![]),
        },
        BoneTimeline {
            label: String::from("bone2"),
            open: false,
            keys: vec![],
            translate: Timeline::new(vec![]),
            rotate: Timeline::new(vec![]),
            scale: Timeline::new(vec![]),
            shear: Timeline::new(vec![]),
        },
    ];

    let name = String::from("some animation");

    Animation { name, bones }
}
