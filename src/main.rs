#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use crate::scene::{ReflectScene, ReflectSceneSpawner};
use crate::ui::AddEditorTab;
use crate::util::anymap::AnyMap;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::reflect::TypeRegistryArc;
use bevy::window::PresentMode;
use bevy::winit::{UpdateMode, WinitSettings};

pub mod anima;
pub mod scene;
pub mod ui;
pub mod util;

//mod workspace;

fn main() {
    crate::util::enable_tracing();

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::CRIMSON))
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(WindowDescriptor {
            //width: 1920.0 / 2.0,
            //height: 1080.0 / 2.0,
            title: String::from("ShaderLab"),
            //mode: WindowMode::Fullscreen,
            decorations: true,
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        })
        .init_resource::<AnyMap>()
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
        .add_plugins_with(DefaultPlugins, |group| group.disable::<LogPlugin>());

    app.add_plugin(crate::scene::component::EditorPlugin);
    app.add_plugin(crate::scene::GizmoPlugin);

    app.add_plugin(self::ui::EditorUiPlugin);

    app.add_editor_tab::<self::scene::FileBrowser>();
    app.add_editor_tab::<self::scene::Hierarchy>();
    app.add_editor_tab::<self::scene::Inspector>();
    app.add_editor_tab::<self::scene::SceneTab>();

    app.add_plugin(self::anima::AnimaPlugin);

    app.add_startup_system(setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut spawner: ResMut<ReflectSceneSpawner>,
    mut scenes: ResMut<Assets<ReflectScene>>,
    type_registry: Res<TypeRegistryArc>,
) {
    use crate::anima::{Animation2d, TimelinePanel};
    use crate::scene::{FileBrowser, Hierarchy, Inspector, SceneTab};
    use crate::ui::*;

    {
        let world = exampe_scene();
        let scene = ReflectScene::from_world(&world, &type_registry);
        let scene = scenes.add(scene);
        spawner.spawn(scene.clone());
        commands.insert_resource(scene);
    }

    trait SpawnTab {
        fn tab<T: Component>(&mut self, icon: char, title: &str, tab: T) -> Tab;

        fn placeholder(&mut self, icon: char, title: &str) -> Tab {
            self.tab(icon, title, PlaceholderTab::default())
        }
    }

    impl<'s, 'w> SpawnTab for Commands<'s, 'w> {
        fn tab<T: Component>(&mut self, icon: char, title: &str, tab: T) -> Tab {
            let entity = self
                .spawn()
                .insert_bundle((EditorPanel::default(), tab))
                .id();
            Tab::new(icon, title, entity)
        }
    }

    let node_tree = commands.placeholder(icon::NODETREE, "Node Tree");

    let scene = SceneTab::spawn(&mut commands, &mut images);

    let hierarchy = commands.tab(icon::OUTLINER, "Hierarchy", Hierarchy::default());
    let inspector = commands.tab(icon::PROPERTIES, "Inspector", Inspector::default());
    let files = commands.tab(icon::FILEBROWSER, "File Browser", FileBrowser::default());

    let assets = commands.placeholder(icon::ASSET_MANAGER, "Asset Manager");

    let anim = Animation2d::spawn(&mut commands, &mut images);

    let timeline = commands.tab(icon::TIME, "Timeline", TimelinePanel::default());

    let root = TreeNode::leaf_with(vec![anim, scene, node_tree]);
    let mut split_tree = SplitTree::new(root);

    let [a, b] = split_tree.split_tabs(NodeIndex::root(), Split::Right, 0.7, vec![inspector]);
    let [_, _] = split_tree.split_tabs(a, Split::Below, 0.8, vec![timeline]);
    let [_, _] = split_tree.split_tabs(b, Split::Below, 0.5, vec![hierarchy, files, assets]);

    commands.insert_resource(split_tree);
}

fn exampe_scene() -> World {
    use crate::scene::component::{ProxyHandle, ProxyMeta, ProxyPointLight, ProxyTransform};
    use crate::ui::icon;
    use bevy::ecs::world::EntityMut;

    fn new<'a>(builder: &'a mut WorldChildBuilder, prefix: &str, counter: usize) -> EntityMut<'a> {
        let icon = match counter % 14 {
            0 => icon::MESH_CONE,
            1 => icon::MESH_PLANE,
            2 => icon::MESH_CYLINDER,
            3 => icon::MESH_ICOSPHERE,
            4 => icon::MESH_CAPSULE,
            5 => icon::MESH_UVSPHERE,
            6 => icon::MESH_CIRCLE,
            7 => icon::MESH_MONKEY,
            8 => icon::MESH_TORUS,
            9 => icon::MESH_CUBE,

            10 => icon::OUTLINER_OB_CAMERA,
            11 => icon::OUTLINER_OB_EMPTY,
            12 => icon::OUTLINER_OB_LIGHT,
            13 => icon::OUTLINER_OB_SPEAKER,

            _ => unreachable!(),
        };

        let mut builder = builder.spawn();
        builder.insert_bundle((
            ProxyMeta::new(icon, format!("{} #{}", prefix, counter)),
            ProxyHandle::<Mesh>::new("models/BoxTextured.glb#Mesh0/Primitive0"),
            ProxyHandle::<StandardMaterial>::new("models/BoxTextured.glb#Material0"),
        ));
        builder
    }

    let mut world = World::new();

    world.spawn().insert_bundle((
        ProxyMeta::new(icon::LIGHT_POINT, "Point Light"),
        ProxyTransform {
            translation: Vec3::new(0.0, 0.0, 10.0),
            ..default()
        },
        ProxyPointLight::default(),
    ));

    let mut counter = 0;

    for _ in 0..2 {
        let icon = icon::MESH_CUBE;
        world
            .spawn()
            .insert_bundle((
                ProxyMeta::new(icon, format!("Root #{}", counter)),
                ProxyTransform {
                    translation: Vec3::new(
                        (counter % 2) as f32,
                        (counter % 3) as f32,
                        (counter % 4) as f32,
                    ),
                    ..default()
                },
                ProxyHandle::<Mesh>::new("models/BoxTextured.glb#Mesh0/Primitive0"),
                ProxyHandle::<StandardMaterial>::new("models/BoxTextured.glb#Material0"),
            ))
            .with_children(|builder| {
                for _ in 0..2 {
                    counter += 1;
                    new(builder, "Child", counter)
                        .insert(ProxyTransform {
                            translation: Vec3::new(
                                (counter % 2) as f32,
                                (counter % 3) as f32,
                                (counter % 4) as f32,
                            ),
                            ..default()
                        })
                        .with_children(|builder| {
                            for _ in 0..2 {
                                counter += 1;
                                new(builder, "Sub Child", counter).insert(ProxyTransform {
                                    translation: Vec3::new(
                                        (counter % 2) as f32,
                                        (counter % 3) as f32,
                                        (counter % 4) as f32,
                                    ),
                                    ..default()
                                });
                            }
                        });
                }
            });
        counter += 1;
    }

    world
}
