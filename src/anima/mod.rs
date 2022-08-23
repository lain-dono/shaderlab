pub mod actions;
pub mod animation;
pub mod armature;
pub mod controller;
pub mod example;
pub mod gizmo;
pub mod grid;
pub mod runtime;
pub mod timeline;
pub mod viewport;

pub mod asset;

pub use self::animation::{
    Animation, BoneTimeline, Interpolation, Keyframe, Timeline, TimelineValue,
};
pub use self::controller::{Controller, PlayControl, PlayState};
pub use self::grid::{Grid, GridViewport};
pub use self::runtime::armature::{Armature, Bone};
pub use self::runtime::math::{Matrix, Transform};
pub use self::timeline::TimelinePanel;
pub use self::viewport::Animation2d;

use crate::ui::{AddEditorTab, EditorPanel, PanelRenderTarget};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::window::WindowId;
use reui::plugin::Recorder;

#[derive(Default)]
pub struct AnimaPlugin;

impl bevy::app::Plugin for AnimaPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugin(reui::plugin::ReuiPlugin)
            .insert_resource(Controller {
                current_time: 0,
                ..default()
            })
            .insert_resource(self::example::armature())
            .insert_resource(self::example::animation())
            .add_system(sync_frame)
            .add_system_to_stage(CoreStage::PostUpdate, draw)
            .add_editor_tab::<TimelinePanel>()
            .add_editor_tab::<Animation2d>();
    }
}

fn sync_frame(mut ctrl: ResMut<Controller>, armature: Res<Armature>, animation: Res<Animation>) {
    ctrl.local_to_world.clear();

    let time = ctrl.current_time as f32;
    for (index, bone) in armature.bones.iter().enumerate() {
        let clip = animation.resolve_bone_tranform(index, time);

        //bone.transform.to_matrix().prepend(clip.to_matrix())
        let transform = bone.transform.mul_transform(clip).to_matrix();

        let parent = bone.parent as usize;
        let parent = *ctrl.local_to_world.get(parent).unwrap_or(&Matrix::IDENTITY);
        ctrl.local_to_world.push(parent.prepend(transform));
    }
}

impl Animation2d {
    pub fn spawn(commands: &mut Commands, images: &mut Assets<Image>) -> crate::ui::Tab {
        use bevy::prelude::*;

        let target = PanelRenderTarget::create_render_target(images);

        let clear_color = bevy::prelude::Color::rgba(0.0, 0.0, 0.0, 0.0);
        let clear_color = ClearColorConfig::Custom(clear_color);

        let entity = commands
            .spawn_bundle(Camera2dBundle {
                camera_2d: Camera2d { clear_color },
                camera: Camera {
                    target,
                    ..default()
                },
                ..default()
            })
            .insert(Self::default())
            .insert(EditorPanel::default())
            .insert(Recorder::default())
            .insert(PanelRenderTarget::default())
            .id();

        crate::ui::Tab::new(crate::ui::icon::VIEW_ORTHO, "Animate 2d", entity)
    }
}

fn draw(
    windows: Res<Windows>,
    time: Res<Time>,
    mut query: Query<(&mut Recorder, &Animation2d, &EditorPanel)>,
) {
    let time = time.seconds_since_startup() as f32;
    let ppi = windows.scale_factor(WindowId::primary()) as f32;
    for (mut recorder, anima, panel) in query.iter_mut() {
        recorder.clear();

        if let Some(frame) = panel.viewport {
            let zoom = anima.grid.zoom;
            let offset = frame.center() - frame.min;
            let offset = offset + anima.grid.offset * egui::vec2(-zoom, zoom);
            let viewport = reui::Transform::compose(offset.x * ppi, offset.y * ppi, 0.0, ppi);

            let mut canvas = self::gizmo::Gizmos::new(&mut recorder, viewport);

            let x_color = reui::Color::bgra(0xFFFF0000);
            let y_color = reui::Color::bgra(0xFF00FF00);

            use std::f32::consts::{FRAC_PI_2, PI};
            let rot = -time;

            canvas.set_transform(0.0, 0.0, rot, 1.0);
            canvas.rotate(x_color);
            canvas.set_transform(45.0, 0.0, rot, 1.0);
            canvas.length(x_color);

            canvas.set_transform(45.0, 20.0, rot, 1.0);
            canvas.pose(x_color);

            canvas.set_transform(70.0, 0.0, rot, 1.0);
            canvas.arrow_translate(x_color);
            canvas.set_transform(70.0, 0.0, rot - FRAC_PI_2, 1.0);
            canvas.arrow_translate(y_color);

            canvas.set_transform(140.0, 0.0, rot, 1.0);
            canvas.arrow_scale(x_color);
            canvas.set_transform(140.0, 0.0, rot - FRAC_PI_2, 1.0);
            canvas.arrow_scale(y_color);

            canvas.set_transform(200.0, 0.0, 0.0, 1.0);
            canvas.shear(x_color, time);
            canvas.set_transform(200.0, 0.0, PI, 1.0);
            canvas.shear(x_color, time);

            canvas.set_transform(200.0, 0.0, FRAC_PI_2, 1.0);
            canvas.shear(y_color, -time);
            canvas.set_transform(200.0, 0.0, -FRAC_PI_2, 1.0);
            canvas.shear(y_color, -time);
        }
    }
}
