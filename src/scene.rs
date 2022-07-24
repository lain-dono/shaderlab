mod asset;
mod gizmo;
mod tab;

pub use self::{
    asset::{
        ReflectEntity, ReflectScene, ReflectSceneLoader, ReflectSceneSpawnError,
        ReflectSceneSpawner, SceneMapping,
    },
    gizmo::{GizmoPlugin, GIZMO_DRIVER},
    tab::{SceneRenderTarget, SceneTab},
};

use bevy::ecs::event::Events;
use bevy::prelude::*;

#[derive(Default)]
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<ReflectScene>()
            .init_asset_loader::<ReflectSceneLoader>()
            .init_resource::<ReflectSceneSpawner>()
            .insert_resource(SceneRenderTarget(None))
            .add_system_to_stage(
                CoreStage::PreUpdate,
                scene_spawner_system.exclusive_system().at_end(),
            )
            .add_system(update_scene_render_target.after(crate::app::ui_root));
    }
}

pub fn scene_spawner_system(world: &mut World) {
    world.resource_scope(|world, mut spawner: Mut<ReflectSceneSpawner>| {
        let scene_asset_events = world.resource::<Events<AssetEvent<ReflectScene>>>();

        let mut handles = Vec::new();
        let spawner = &mut *spawner;
        for event in spawner.event_reader.iter(scene_asset_events) {
            if let AssetEvent::Modified { handle } = event {
                if spawner.scenes.contains_key(handle) {
                    handles.push(handle.clone_weak());
                }
            }
        }

        spawner.despawn_queued(world).unwrap();
        spawner.spawn_queued(world).unwrap();
        spawner.update_spawned(world, &handles).unwrap();
        spawner.set_scene_instance_parent_sync(world);
    });
}

pub fn update_scene_render_target(
    mut tree: ResMut<crate::app::SplitTree>,
    mut egui_context: ResMut<crate::shell::EguiContext>,
    scene_render_target: Res<SceneRenderTarget>,
    mut images: ResMut<Assets<Image>>,
    mut camera: Query<
        (&mut Transform, &mut PerspectiveProjection),
        With<bevy::render::camera::Camera>,
    >,
) {
    let [ctx] = egui_context.ctx_mut([bevy::window::WindowId::primary()]);

    if let Some(handle) = scene_render_target.0.as_ref() {
        if let Some(image) = images.get_mut(handle) {
            if let Some((viewport, tab)) = tree.find_active::<SceneTab>() {
                let width = (viewport.width() * ctx.pixels_per_point()) as u32;
                let height = (viewport.height() * ctx.pixels_per_point()) as u32;

                image.resize(wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    ..default()
                });

                tab.texture_id = Some(egui_context.add_image(handle.clone_weak()));

                for (mut transform, mut projection) in camera.iter_mut() {
                    *transform = tab.camera_view;
                    *projection = tab.camera_proj.clone();
                }
            }
        }
    }
}
