pub mod actions;
pub mod animation;
pub mod armature;
pub mod controller;
pub mod example;
pub mod grid;
pub mod math;
pub mod timeline;
pub mod viewport;

pub use self::animation::{Animation, BoneTimeline, Curve, Keyframe};
pub use self::armature::{Armature, Bone};
pub use self::controller::{Controller, PlayControl};
pub use self::math::Matrix;
pub use self::viewport::Animation2d;

use crate::ui::{AddEditorTab, EditorTab, Style};
use bevy::ecs::system::lifetimeless::{SRes, SResMut};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::{default, Entity, Res, ResMut, Time};

#[derive(Default, bevy::prelude::Component)]
pub struct TimelinePanel;

#[derive(Default)]
pub struct Anima;

impl bevy::app::Plugin for Anima {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_startup_system(setup)
            .add_editor_tab::<TimelinePanel>()
            .add_editor_tab::<Animation2d>()
            .add_system(animate_rot);
    }
}

pub fn setup(mut commands: bevy::prelude::Commands) {
    commands.insert_resource(self::example::armature());
    commands.insert_resource(self::example::animation());
    commands.insert_resource(Controller {
        current_time: 8,
        max_time: 15,
        ..default()
    });
}

impl EditorTab for TimelinePanel {
    type Param = (SRes<Style>, SResMut<Animation>, SResMut<Controller>);

    fn ui<'w>(
        &mut self,
        ui: &mut egui::Ui,
        _entity: Entity,
        (style, animation, controller): &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        self::timeline::run_ui(controller, ui, style, animation);
    }
}

impl EditorTab for Animation2d {
    type Param = (SRes<Style>, SResMut<Armature>, SResMut<Controller>);

    fn ui<'w>(
        &mut self,
        ui: &mut egui::Ui,
        _entity: Entity,
        (style, armature, controller): &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        self.run_ui(ui, style, armature, controller);
    }
}

fn animate_rot(time: Res<Time>, mut armature: ResMut<Armature>) {
    for bone in &mut armature.bones {
        bone.rotation = time.seconds_since_startup() as f32;
        bone.rotation %= std::f32::consts::TAU;
    }
}
