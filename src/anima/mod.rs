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

use crate::app::EditorPanel;
use crate::style::Style;
use bevy::prelude::{default, Entity, Query, Res, ResMut, Time, With};
use bevy::window::WindowId;

#[derive(Default, bevy::prelude::Component)]
pub struct TimelinePanel;

#[derive(Default)]
pub struct Anima;

impl bevy::app::Plugin for Anima {
    fn build(&self, app: &mut bevy::app::App) {
        use bevy::prelude::*;

        app.add_startup_system(setup)
            .add_system(timeline_panel.after(crate::app::ui_root_tabs_sync))
            .add_system(viewport_panel.after(crate::app::ui_root_tabs_sync))
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

pub fn timeline_panel(
    mut context: ResMut<crate::shell::EguiContext>,
    style: Res<Style>,
    query: Query<(Entity, &EditorPanel), With<TimelinePanel>>,
    mut animation: ResMut<Animation>,
    mut controller: ResMut<Controller>,
) {
    let [ctx] = context.ctx_mut([WindowId::primary()]);
    for (entity, viewport) in query.iter() {
        if let Some(viewport) = viewport.viewport {
            let id = egui::Id::new("AnimaTimeline").with(entity);
            let mut ui = egui::Ui::new(
                ctx.clone(),
                egui::LayerId::background(),
                id,
                viewport,
                viewport,
            );
            self::timeline::run_ui(&mut controller, &mut ui, &style, &mut animation);
        }
    }
}

pub fn viewport_panel(
    mut context: ResMut<crate::shell::EguiContext>,
    style: Res<Style>,
    mut query: Query<(Entity, &EditorPanel, &mut Animation2d)>,
    mut armature: ResMut<Armature>,
    mut controller: ResMut<Controller>,
) {
    let [ctx] = context.ctx_mut([WindowId::primary()]);
    for (entity, viewport, mut anima) in query.iter_mut() {
        if let Some(viewport) = viewport.viewport {
            let id = egui::Id::new("AnimaViewport").with(entity);
            let mut ui = egui::Ui::new(
                ctx.clone(),
                egui::LayerId::background(),
                id,
                viewport,
                viewport,
            );
            anima.run_ui(&mut ui, &style, &mut armature, &mut controller);
        }
    }
}

fn animate_rot(time: Res<Time>, mut armature: ResMut<Armature>) {
    for bone in &mut armature.bones {
        bone.rotation = time.seconds_since_startup() as f32;
        bone.rotation %= std::f32::consts::TAU;
    }
}
