#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use crate::app::{AddEditorTab, EditorStage};
use crate::scene::{ReflectScene, ReflectSceneSpawner};
use crate::util::anymap::AnyMap;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::reflect::TypeRegistryArc;
use bevy::render::{render_graph::RenderGraph, RenderApp};
use bevy::window::{PresentMode, WindowId};
use bevy::winit::{UpdateMode, WinitSettings};

mod anima;
mod app;
mod component;
mod context;
mod field;
mod icon;
mod panel;
mod scene;
mod shell;
mod style;
mod util;

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
        .add_plugins_with(DefaultPlugins, |group| group.disable::<LogPlugin>())
        .add_plugin(crate::component::EditorPlugin)
        .add_plugin(shell::EguiPlugin)
        .add_plugin(scene::GizmoPlugin)
        .add_startup_system(setup);

    app.add_plugin(self::app::EditorUiPlugin);

    app.add_editor_tab::<self::panel::PlaceholderTab>();

    app.add_system_to_stage(EditorStage::Tabs, self::panel::FileBrowser::system);
    app.add_system_to_stage(EditorStage::Tabs, self::panel::Inspector::system);
    app.add_system_to_stage(EditorStage::Tabs, self::panel::Hierarchy::system);
    app.add_system_to_stage(EditorStage::Tabs, self::scene::SceneTab::system);
    app.add_plugin(self::anima::Anima);

    {
        let render_app = app.sub_app_mut(RenderApp);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();

        // add egui nodes
        shell::setup_pipeline(&mut graph, WindowId::primary(), "ui_root");
    }

    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut context: ResMut<crate::shell::EguiContext>,
    mut spawner: ResMut<ReflectSceneSpawner>,
    mut scenes: ResMut<Assets<ReflectScene>>,
    type_registry: Res<TypeRegistryArc>,
) {
    commands.insert_resource(crate::style::Style::default());

    init_split_tree(&mut commands, &mut images);

    let world = exampe_scene();
    let scene = ReflectScene::from_world(&world, &type_registry);
    let scene = scenes.add(scene);
    spawner.spawn(scene.clone());
    commands.insert_resource(scene);

    {
        let [ctx] = context.ctx_mut([bevy::window::WindowId::primary()]);
        ctx.set_fonts(fonts_with_blender());
    }
}

pub struct SelectedEntity(pub Option<Entity>);

pub fn fonts_with_blender() -> egui::FontDefinitions {
    let font = egui::FontData::from_static(include_bytes!("icon.ttf"));

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("blender".to_owned(), font);
    fonts.families.insert(
        egui::FontFamily::Name("blender".into()),
        vec!["Hack".to_owned(), "blender".into()],
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .push("blender".to_owned());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("blender".to_owned());

    fonts
}

fn exampe_scene() -> World {
    use crate::component::{ProxyHandle, ProxyMeta, ProxyPointLight, ProxyTransform};
    use bevy::ecs::world::EntityMut;

    fn new<'a>(builder: &'a mut WorldChildBuilder, prefix: &str, counter: usize) -> EntityMut<'a> {
        let icon = match counter % 14 {
            0 => crate::icon::MESH_CONE,
            1 => crate::icon::MESH_PLANE,
            2 => crate::icon::MESH_CYLINDER,
            3 => crate::icon::MESH_ICOSPHERE,
            4 => crate::icon::MESH_CAPSULE,
            5 => crate::icon::MESH_UVSPHERE,
            6 => crate::icon::MESH_CIRCLE,
            7 => crate::icon::MESH_MONKEY,
            8 => crate::icon::MESH_TORUS,
            9 => crate::icon::MESH_CUBE,

            10 => crate::icon::OUTLINER_OB_CAMERA,
            11 => crate::icon::OUTLINER_OB_EMPTY,
            12 => crate::icon::OUTLINER_OB_LIGHT,
            13 => crate::icon::OUTLINER_OB_SPEAKER,

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
    world.insert_resource(SelectedEntity(None));

    world.spawn().insert_bundle((
        ProxyMeta::new(crate::icon::LIGHT_POINT, "Point Light"),
        ProxyTransform {
            translation: Vec3::new(0.0, 0.0, 10.0),
            ..default()
        },
        ProxyPointLight::default(),
    ));

    let mut counter = 0;

    for _ in 0..2 {
        let icon = crate::icon::MESH_CUBE;
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

fn init_split_tree(commands: &mut Commands, images: &mut Assets<Image>) {
    use crate::anima::{Animation2d, TimelinePanel};
    use crate::app::*;
    use crate::panel::{FileBrowser, Hierarchy, Inspector, PlaceholderTab};
    use crate::scene::SceneTab;

    trait SpawnTab {
        fn spawn_tab<T: Component>(&mut self, icon: char, title: &str, tab: T) -> Tab;

        fn spawn_placeholder(&mut self, icon: char, title: &str) -> Tab {
            self.spawn_tab(icon, title, PlaceholderTab::default())
        }
    }

    impl<'s, 'w> SpawnTab for Commands<'s, 'w> {
        fn spawn_tab<T: Component>(&mut self, icon: char, title: &str, tab: T) -> Tab {
            let entity = self
                .spawn()
                .insert_bundle((EditorPanel::default(), tab))
                .id();
            Tab::new(icon, title, entity)
        }
    }

    let node_tree = commands.spawn_placeholder(icon::NODETREE, "Node Tree");

    let scene_entity = SceneTab::spawn(commands, images);
    let scene = Tab::new(icon::VIEW3D, "Scene", scene_entity);

    let hierarchy = commands.spawn_tab(icon::OUTLINER, "Hierarchy", Hierarchy::default());
    let inspector = commands.spawn_tab(icon::PROPERTIES, "Inspector", Inspector::default());
    let files = commands.spawn_tab(icon::FILEBROWSER, "File Browser", FileBrowser::default());

    let assets = commands.spawn_placeholder(icon::ASSET_MANAGER, "Asset Manager");

    let anim = commands.spawn_tab(icon::VIEW_ORTHO, "Animate 2d", Animation2d::default());
    let timeline = commands.spawn_tab(icon::TIME, "Timeline", TimelinePanel::default());

    let root = TreeNode::leaf_with(vec![anim, scene, node_tree]);
    let mut split_tree = SplitTree::new(root);

    let [a, b] = split_tree.split_tabs(NodeIndex::root(), Split::Right, 0.7, vec![inspector]);
    let [_, _] = split_tree.split_tabs(a, Split::Below, 0.8, vec![timeline]);
    let [_, _] = split_tree.split_tabs(b, Split::Below, 0.5, vec![hierarchy, files, assets]);

    commands.insert_resource(split_tree);
}
