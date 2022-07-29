pub mod actions;
pub mod animation;
pub mod armature;
pub mod example;
pub mod grid;
pub mod math;
pub mod timeline;
pub mod viewport;

pub use self::animation::{Animation, BoneTimeline, Curve, Keyframe};
pub use self::armature::{Armature, Bone};
pub use self::math::Matrix;
pub use self::timeline::TimelinePanel;
pub use self::viewport::Animation2d;

#[derive(Default)]
pub struct Controller {
    /// World matrices for bones.
    pub world: Vec<Matrix>,
}

impl Controller {
    pub fn collect_world_transform(&mut self, armature: &Armature) {
        self.world.clear();

        for bone in &armature.bones {
            let local = bone.local_matrix();
            let parent = bone.parent as usize;
            let parent = *self.world.get(parent).unwrap_or(&Matrix::IDENTITY);
            self.world.push(parent.prepend(local));
        }
    }

    pub fn animate(&mut self, armature: &Armature, animaton: &Animation, frame: usize) {
        self.world.clear();

        for bone in &armature.bones {
            let local = bone.local_matrix();
            let parent = bone.parent as usize;
            let parent = *self.world.get(parent).unwrap_or(&Matrix::IDENTITY);
            self.world.push(parent.prepend(local));
        }
    }
}

#[derive(Default)]
pub struct Anima;

impl bevy::app::Plugin for Anima {
    fn build(&self, app: &mut bevy::app::App) {
        //use bevy::prelude::*;

        app.add_startup_system(setup);
    }
}

pub fn setup(mut commands: bevy::prelude::Commands) {
    commands.insert_resource(self::example::armature());
    commands.insert_resource(self::example::animation());
    commands.insert_resource(Controller::default());
}
