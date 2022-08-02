pub mod actions;
pub mod animation;
pub mod armature;
pub mod controller;
pub mod example;
pub mod grid;
pub mod math;
pub mod timeline;
pub mod viewport;

pub use self::animation::{Animation, BoneTimeline, Interpolation, Keyframe};
pub use self::armature::{Armature, Bone};
pub use self::controller::{Controller, PlayControl, PlayState};
pub use self::math::Matrix;
pub use self::timeline::TimelinePanel;
pub use self::viewport::Animation2d;

use crate::ui::AddEditorTab;
use bevy::prelude::{default, Res, ResMut};

#[derive(Default)]
pub struct Anima;

impl bevy::app::Plugin for Anima {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(Controller {
            current_time: 8,
            max_time: 15,
            ..default()
        })
        .insert_resource(self::example::armature())
        .insert_resource(self::example::animation())
        .add_system(sync_frame)
        .add_editor_tab::<TimelinePanel>()
        .add_editor_tab::<Animation2d>();
    }
}

fn sync_frame(mut ctrl: ResMut<Controller>, armature: Res<Armature>, animation: Res<Animation>) {
    ctrl.world.clear();

    let time = ctrl.current_time as f32;
    for (index, bone) in armature.bones.iter().enumerate() {
        let mut transform = transform(bone.rotation, bone.location, bone.scale, bone.shear);
        if let Some(anim_bone) = animation.bones.get(index) {
            transform = transform.prepend(anim_bone.resolve(time));
        }

        let parent = bone.parent as usize;
        let parent = *ctrl.world.get(parent).unwrap_or(&Matrix::IDENTITY);
        ctrl.world.push(parent.prepend(transform));
    }
}

pub fn transform(
    rotation: f32,
    location: egui::Pos2,
    scale: egui::Vec2,
    shear: egui::Vec2,
) -> Matrix {
    let (sx, cx) = (rotation - shear.x).sin_cos();
    let (sy, cy) = (rotation + shear.y).sin_cos();
    let sx = -sx;

    Matrix {
        a: cy * scale.x,
        b: sy * scale.x,
        c: sx * scale.y,
        d: cx * scale.y,
        tx: location.x,
        ty: location.y,
    }
}
