#![allow(clippy::forget_non_drop)]

use bevy::core_pipeline::{
    draw_3d_graph, node, AlphaMask3d, Opaque3d, RenderTargetClearColors, Transparent3d,
};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::{
    camera::{ActiveCamera, Camera, CameraTypePlugin, RenderTarget},
    render_graph::{
        Node, NodeRunError, RenderGraph, RenderGraphContext, RenderGraphError, SlotValue,
    },
    render_phase::RenderPhase,
    render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    {renderer::RenderContext, view::RenderLayers, RenderApp, RenderStage},
};
use bevy::window::{PresentMode, WindowId};
use bevy::winit::{UpdateMode, WinitSettings};

pub mod app;
pub mod blender;
pub mod global;
pub mod panel;
pub mod shell;
pub mod style;
pub mod util;

#[derive(Component, Default)]
pub struct FirstPassCamera;

// The name of the final node of the first pass.
pub const FIRST_PASS_DRIVER: &str = "first_pass_driver";

fn main() {
    crate::util::enable_tracing();

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::CRIMSON))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: String::from("ShaderLab"),
            ..default()
        })
        .add_plugins_with(DefaultPlugins, |group| group.disable::<LogPlugin>())
        .register_type::<crate::global::Icon>()
        .add_plugin(shell::EguiPlugin)
        .add_plugin(CameraTypePlugin::<FirstPassCamera>::default())
        // Optimal power saving and present mode settings for desktop apps.
        .insert_resource(WinitSettings {
            return_from_run: true,
            //focused_mode: UpdateMode::Continuous,
            focused_mode: UpdateMode::Reactive {
                max_wait: std::time::Duration::from_secs_f64(1.0 / 60.0),
            },
            unfocused_mode: UpdateMode::ReactiveLowPower {
                max_wait: std::time::Duration::from_secs(60),
            },
        })
        .insert_resource(WindowDescriptor {
            present_mode: PresentMode::Mailbox,
            ..default()
        })
        .add_startup_system(setup)
        .add_startup_system(crate::global::setup)
        .add_system(crate::app::ui_root)
        .add_system(scene_force_set_changed)
        .insert_resource(crate::panel::scene::SceneRenderTarget(None))
        .add_system(crate::panel::scene::update_scene_render_target.after(crate::app::ui_root))
        .add_system(cube_rotator_system)
        .add_system(rotator_system)
        .add_system_to_stage(CoreStage::First, extract_editable_scene.exclusive_system());

    init_graph(app.sub_app_mut(RenderApp)).unwrap();

    app.run();
}

fn extract_editable_scene(world: &mut World) {
    use crate::panel::scene::SceneRenderTarget;
    use bevy::reflect::TypeRegistryArc;
    use bevy::render::camera::Camera3d;
    use bevy::scene::SceneSpawnError;
    //use bevy::ecs::reflect::ReflectMapEntities;

    let transform = {
        let transform = world
            .get_resource::<ActiveCamera<Camera3d>>()
            .and_then(|camera| camera.get())
            .and_then(|camera| world.get::<Transform>(camera));

        match transform {
            Some(transform) => *transform,
            None => return,
        }
    };

    world.clear_entities();

    let type_registry = world.resource::<TypeRegistryArc>().clone();
    let type_registry = type_registry.read();

    world.resource_scope(|world, scene: Mut<Scene>| {
        for archetype in scene.world.archetypes().iter() {
            for scene_entity in archetype.entities() {
                let entity = world.spawn().id();
                for component_id in archetype.components() {
                    let component_info = scene
                        .world
                        .components()
                        .get_info(component_id)
                        .expect("component_ids in archetypes should have ComponentInfo");

                    let reflect_component = type_registry
                        .get(component_info.type_id().unwrap())
                        .ok_or_else(|| SceneSpawnError::UnregisteredType {
                            type_name: component_info.name().to_string(),
                        })
                        .and_then(|registration| {
                            registration.data::<ReflectComponent>().ok_or_else(|| {
                                SceneSpawnError::UnregisteredComponent {
                                    type_name: component_info.name().to_string(),
                                }
                            })
                        });

                    match reflect_component {
                        Ok(reflect_component) => reflect_component.copy_component(
                            &scene.world,
                            world,
                            *scene_entity,
                            entity,
                        ),
                        Err(err) => bevy::log::error!("respawn: {:?}", err),
                    }
                }
            }
        }

        /*
        for registration in type_registry.iter() {
            if let Some(map_entities_reflect) = registration.data::<ReflectMapEntities>() {
                map_entities_reflect
                    .map_entities(world, &instance_info.entity_map)
                    .unwrap();
            }
        }
        */

        /*
        self.spawned_instances.insert(instance_id, instance_info);
        let spawned = self
            .spawned_scenes
            .entry(scene_handle)
            .or_insert_with(Vec::new);
        spawned.push(instance_id);
        Ok(instance_id)
        */
    });

    if let Some(handle) = world
        .get_resource::<SceneRenderTarget>()
        .and_then(|s| s.0.as_ref())
        .cloned()
    {
        let target = RenderTarget::Image(handle);

        world
            .resource_mut::<RenderTargetClearColors>()
            .insert(target.clone(), Color::CRIMSON);

        world.spawn().insert_bundle(PerspectiveCameraBundle {
            camera: Camera {
                target,
                ..default()
            },
            transform,
            ..default()
        });
    }
}

fn scene_force_set_changed(handle: Option<ResMut<Handle<DynamicScene>>>) {
    if let Some(mut handle) = handle {
        handle.set_changed();
    }
    /*
    if let Some(handle) = world.get_resource::<Handle<DynamicScene>>().cloned() {
        world.resource_scope(|world, mut spawner: Mut<SceneSpawner>| {
            spawner.update_spawned_scenes(world, &[handle]).unwrap();
        });
    }
    */
}

fn init_graph(render_app: &mut App) -> Result<(), RenderGraphError> {
    // This will add 3D render phases for the new camera.
    render_app.add_system_to_stage(RenderStage::Extract, extract_first_pass_camera_phases);

    let driver = FirstPassCameraDriver::new(&mut render_app.world);
    let mut graph = render_app.world.resource_mut::<RenderGraph>();

    // add egui nodes
    shell::setup_pipeline(&mut graph, WindowId::primary(), "ui_root");

    // Add a node for the first pass.
    graph.add_node(FIRST_PASS_DRIVER, driver);

    // The first pass's dependencies include those of the main pass.
    graph.add_node_edge(node::MAIN_PASS_DEPENDENCIES, FIRST_PASS_DRIVER)?;

    // Insert the first pass node: CLEAR_PASS_DRIVER -> FIRST_PASS_DRIVER -> MAIN_PASS_DRIVER
    graph.add_node_edge(node::CLEAR_PASS_DRIVER, FIRST_PASS_DRIVER)?;
    graph.add_node_edge(FIRST_PASS_DRIVER, node::MAIN_PASS_DRIVER)?;

    Ok(())
}

// Add 3D render phases for FIRST_PASS_CAMERA.
fn extract_first_pass_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<FirstPassCamera>>,
) {
    if let Some(entity) = active.get() {
        commands.get_or_spawn(entity).insert_bundle((
            RenderPhase::<Opaque3d>::default(),
            RenderPhase::<AlphaMask3d>::default(),
            RenderPhase::<Transparent3d>::default(),
        ));
    }
}

// A node for the first pass camera that runs draw_3d_graph with this camera.
struct FirstPassCameraDriver {
    query: QueryState<Entity, With<FirstPassCamera>>,
}

impl FirstPassCameraDriver {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}

impl Node for FirstPassCameraDriver {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        for camera in self.query.iter_manual(world) {
            graph.run_sub_graph(draw_3d_graph::NAME, vec![SlotValue::Entity(camera)])?;
        }
        Ok(())
    }
}

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut clear_colors: ResMut<RenderTargetClearColors>,
) {
    let (first_pass_layer, image_handle) = {
        let size = Extent3d {
            width: 32,
            height: 32,
            ..default()
        };

        // This is the texture that will be rendered to.
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..default()
        };

        // fill image.data with zeroes
        image.resize(size);

        let image_handle = images.add(image);

        // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
        let first_pass_layer = RenderLayers::layer(1);

        // First pass camera
        let render_target = RenderTarget::Image(image_handle.clone());
        clear_colors.insert(render_target.clone(), Color::WHITE);

        commands
            .spawn_bundle(PerspectiveCameraBundle::<FirstPassCamera> {
                camera: Camera {
                    target: render_target,
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                    .looking_at(Vec3::default(), Vec3::Y),
                ..PerspectiveCameraBundle::new()
            })
            .insert(first_pass_layer);

        // NOTE: omitting the RenderLayers component for this camera may cause a validation error:
        //
        // thread 'main' panicked at 'wgpu error: Validation Error
        //
        //    Caused by:
        //        In a RenderPass
        //          note: encoder = `<CommandBuffer-(0, 1, Metal)>`
        //        In a pass parameter
        //          note: command buffer = `<CommandBuffer-(0, 1, Metal)>`
        //        Attempted to use texture (5, 1, Metal) mips 0..1 layers 0..1 as a combination of COLOR_TARGET within a usage scope.
        //
        // This happens because the texture would be written and read in the same frame, which is not allowed.
        // So either render layers must be used to avoid this, or the texture must be double buffered.

        (first_pass_layer, image_handle)
    };

    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 4.0 }));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // The cube that will be rendered to the texture.
    commands
        .spawn_bundle(PbrBundle {
            mesh: cube_handle,
            material: cube_material_handle,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        })
        .insert(FirstPassCube)
        .insert(first_pass_layer);

    // Light
    // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    });

    let cube_size = 4.0;
    let cube_handle = meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size)));

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // Main pass cube, with material containing the rendered first pass texture.
    commands
        .spawn_bundle(PbrBundle {
            mesh: cube_handle,
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                rotation: Quat::from_rotation_x(-std::f32::consts::PI / 5.0),
                ..default()
            },
            ..default()
        })
        .insert(MainPassCube);

    // The main pass camera.
    {
        let image_handle =
            crate::panel::scene::SceneRenderTarget::insert(&mut commands, &mut images);

        let render_target = RenderTarget::Image(image_handle);
        clear_colors.insert(render_target.clone(), Color::CRIMSON);

        commands.spawn_bundle(PerspectiveCameraBundle {
            camera: Camera {
                target: render_target,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..default()
        });
    }
}

/// Rotates the inner cube (first pass)
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<FirstPassCube>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_x(1.5 * time.delta_seconds());
        transform.rotation *= Quat::from_rotation_z(1.3 * time.delta_seconds());
    }
}

/// Rotates the outer cube (main pass)
fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<MainPassCube>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_x(1.0 * time.delta_seconds());
        transform.rotation *= Quat::from_rotation_y(0.7 * time.delta_seconds());
    }
}
