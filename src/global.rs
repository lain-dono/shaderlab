use crate::asset::{ReflectScene, ReflectSceneSpawner};
use bevy::prelude::*;
use bevy::reflect::TypeRegistryArc;

pub struct SelectedEntity(pub Option<Entity>);

pub fn fonts_with_blender() -> egui::FontDefinitions {
    let font = egui::FontData::from_static(include_bytes!("blender.ttf"));

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
    use crate::asset::{ProxyHandle, ProxyMeta, ProxyTransform};
    use bevy::ecs::world::EntityMut;

    fn new<'a>(builder: &'a mut WorldChildBuilder, prefix: &str, counter: usize) -> EntityMut<'a> {
        let icon = match counter % 14 {
            0 => crate::blender::MESH_CONE,
            1 => crate::blender::MESH_PLANE,
            2 => crate::blender::MESH_CYLINDER,
            3 => crate::blender::MESH_ICOSPHERE,
            4 => crate::blender::MESH_CAPSULE,
            5 => crate::blender::MESH_UVSPHERE,
            6 => crate::blender::MESH_CIRCLE,
            7 => crate::blender::MESH_MONKEY,
            8 => crate::blender::MESH_TORUS,
            9 => crate::blender::MESH_CUBE,

            10 => crate::blender::OUTLINER_OB_CAMERA,
            11 => crate::blender::OUTLINER_OB_EMPTY,
            12 => crate::blender::OUTLINER_OB_LIGHT,
            13 => crate::blender::OUTLINER_OB_SPEAKER,

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

    let mut counter = 0;

    for _ in 0..2 {
        let icon = crate::blender::MESH_CUBE;
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
    use crate::blender;
    use crate::panel::{FileBrowser, Hierarchy, Inspector, PlaceholderTab, SceneTab};

    type NodeTodo = PlaceholderTab;

    let node_tree = PlaceholderTab::default();
    let node_tree = Tab::new(blender::NODETREE, "Node Tree", node_tree);
    let scene = Tab::new(blender::VIEW3D, "Scene", SceneTab::default());

    let hierarchy = Tab::new(blender::OUTLINER, "Hierarchy", Hierarchy::default());
    let inspector = Tab::new(blender::PROPERTIES, "Inspector", Inspector::default());

    let files = Tab::new(blender::FILEBROWSER, "File Browser", FileBrowser::default());
    let assets = Tab::new(blender::ASSET_MANAGER, "Asset Manager", NodeTodo::default());

    let root = TreeNode::leaf_with(vec![scene, node_tree]);
    let mut split_tree = SplitTree::new(root);

    let [a, b] = split_tree.split_tabs(NodeIndex::root(), Split::Right, vec![inspector]);
    let [_, _] = split_tree.split_tabs(a, Split::Below, vec![files, assets]);
    let [_, _] = split_tree.split_tabs(b, Split::Above, vec![hierarchy]);

    split_tree
}
