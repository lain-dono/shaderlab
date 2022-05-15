use crate::asset::{ReflectScene, ReflectSceneSpawner};
use bevy::prelude::*;
use bevy::reflect::TypeRegistryArc;

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

pub fn setup(
    mut commands: Commands,
    mut context: ResMut<crate::shell::EguiContext>,
    mut spawner: ResMut<ReflectSceneSpawner>,
    mut scenes: ResMut<Assets<ReflectScene>>,
    type_registry: Res<TypeRegistryArc>,
) {
    commands.insert_resource(crate::style::Style::default());
    commands.insert_resource(init_split_tree());

    let world = exampe_scene();
    let scene = ReflectScene::from_world(&world, &type_registry);
    let scene = scenes.add(scene);
    spawner.spawn(scene.clone());
    commands.insert_resource(scene);

    {
        let [ctx] = context.ctx_mut([bevy::window::WindowId::primary()]);
        ctx.set_fonts(crate::global::fonts_with_blender());
    }
}

fn init_split_tree() -> crate::app::SplitTree {
    use crate::app::*;
    use crate::icon;
    use crate::panel::{FileBrowser, Hierarchy, Inspector, PlaceholderTab, SceneTab};

    type NodeTodo = PlaceholderTab;

    let node_tree = PlaceholderTab::default();
    let node_tree = Tab::new(icon::NODETREE, "Node Tree", node_tree);
    let scene = Tab::new(icon::VIEW3D, "Scene", SceneTab::default());

    let hierarchy = Tab::new(icon::OUTLINER, "Hierarchy", Hierarchy::default());
    let inspector = Tab::new(icon::PROPERTIES, "Inspector", Inspector::default());

    let files = Tab::new(icon::FILEBROWSER, "File Browser", FileBrowser::default());
    let assets = Tab::new(icon::ASSET_MANAGER, "Asset Manager", NodeTodo::default());

    let root = TreeNode::leaf_with(vec![scene, node_tree]);
    let mut split_tree = SplitTree::new(root);

    let [a, b] = split_tree.split_tabs(NodeIndex::root(), Split::Left, 0.3, vec![inspector]);
    let [_, _] = split_tree.split_tabs(a, Split::Below, 0.7, vec![files, assets]);
    let [_, _] = split_tree.split_tabs(b, Split::Below, 0.5, vec![hierarchy]);

    split_tree
}
