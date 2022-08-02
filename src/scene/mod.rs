pub mod asset;
pub mod component;
pub mod context;
pub mod filebrowser;
pub mod gizmo;
pub mod hierarchy;
pub mod inspector;
pub mod tab;

pub use self::{
    asset::{
        ReflectEntity, ReflectScene, ReflectSceneLoader, ReflectSceneSpawnError,
        ReflectSceneSpawner, SceneMapping,
    },
    filebrowser::FileBrowser,
    gizmo::{GizmoPlugin, GIZMO_DRIVER},
    hierarchy::Hierarchy,
    inspector::Inspector,
    tab::SceneTab,
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
            .add_system_to_stage(
                CoreStage::PreUpdate,
                scene_spawner_system.exclusive_system().at_end(),
            )
            .add_system(update_scene_render_target.after(crate::ui::app::ui_root));
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

use bevy::render::camera::{Camera, RenderTarget};

pub fn update_scene_render_target(
    mut context: ResMut<crate::ui::shell::EguiContext>,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<(&mut SceneTab, &crate::ui::EditorPanel, &Camera)>,
) {
    let [ctx] = context.ctx_mut([bevy::window::WindowId::primary()]);
    let ppi = ctx.pixels_per_point();

    for (mut scene, tab, camera) in query.iter_mut() {
        let handle = if let RenderTarget::Image(handle) = &camera.target {
            handle
        } else {
            continue;
        };

        if let Some(image) = images.get_mut(handle) {
            if let Some(viewport) = tab.viewport {
                let width = (viewport.width() * ppi) as u32;
                let height = (viewport.height() * ppi) as u32;
                scene.texture_id = Some(context.add_image(handle.clone_weak()));

                image.resize(wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    ..default()
                });
            }
        }
    }
}
