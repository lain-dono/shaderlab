use bevy::prelude::*;
use std::borrow::Cow;

pub struct SelectedEntity(pub Option<Entity>);

#[derive(Bundle, Default)]
pub struct EditorBundle {
    pub icon: Icon,
    pub name: Name,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
}

impl EditorBundle {
    pub fn new(icon: char, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            icon: Icon::new(icon),
            name: Name::new(name),
            ..default()
        }
    }
}

#[derive(Clone, Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct Icon {
    icon: u32,
}

impl Icon {
    pub fn new(icon: char) -> Self {
        Self { icon: icon as u32 }
    }

    pub fn get(&self) -> char {
        char::from_u32(self.icon).unwrap()
    }

    pub fn set(&mut self, icon: char) {
        self.icon = icon as u32;
    }
}

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

fn exampe_scene(
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Scene {
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

        builder.spawn_bundle(EditorBundle::new(icon, format!("{} #{}", prefix, counter)))
    }

    let mut world = World::new();
    world.insert_resource(SelectedEntity(None));

    let mut counter = 0;

    for _ in 0..4 {
        let icon = crate::blender::MESH_CUBE;
        world
            .spawn()
            .insert_bundle(EditorBundle::new(icon, format!("Root #{}", counter)))
            .with_children(|builder| {
                for _ in 0..4 {
                    counter += 1;
                    new(builder, "Child", counter).with_children(|builder| {
                        for _ in 0..4 {
                            counter += 1;
                            new(builder, "Sub Child", counter);
                        }
                    });
                }
            });
        counter += 1;
    }

    world.insert_resource(crate::style::Style::default());

    {
        // Light
        // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
        world
            .spawn()
            .insert_bundle(PointLightBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
                ..default()
            })
            .insert_bundle((Icon::new(crate::blender::LIGHT), Name::new("Point Light")));

        let cube_size = 4.0;
        let cube_handle = meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size)));

        // This material has the texture that has been rendered.
        let material_handle = materials.add(StandardMaterial {
            //base_color_texture: Some(image_handle),
            reflectance: 0.02,
            unlit: false,
            ..default()
        });

        // Main pass cube, with material containing the rendered first pass texture.
        world
            .spawn()
            .insert_bundle(PbrBundle {
                mesh: cube_handle,
                material: material_handle,
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 1.5),
                    rotation: Quat::from_rotation_x(-std::f32::consts::PI / 5.0),
                    ..default()
                },
                ..default()
            })
            .insert_bundle((Icon::new(crate::blender::MESH_CUBE), Name::new("Cube")));

        //.insert(MainPassCube);
    }

    Scene { world }
}

pub fn setup(
    mut commands: Commands,
    mut context: ResMut<crate::shell::EguiContext>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(crate::style::Style::default());
    commands.insert_resource(init_split_tree());
    commands.insert_resource(exampe_scene(&mut meshes, &mut materials));

    {
        let [ctx] = context.ctx_mut([bevy::window::WindowId::primary()]);
        ctx.set_fonts(crate::global::fonts_with_blender());
    }
}

fn init_split_tree() -> crate::app::SplitTree {
    use crate::app::*;
    use crate::blender;
    use crate::panel::hierarchy::Hierarchy;
    use crate::panel::inspector::Inspector;
    use crate::panel::placeholder::PlaceholderTab;
    use crate::panel::scene::SceneTab;

    type NodeTodo = PlaceholderTab;

    let node_tree = PlaceholderTab::default();
    let node_tree = Tab::new(
        blender::NODETREE,
        "Node Tree",
        node_tree,
        Schedule::default(),
    );
    let scene = Tab::new(
        blender::VIEW3D,
        "Scene",
        SceneTab::default(),
        Schedule::default(),
    );
    let material = Tab::new(
        blender::MATERIAL,
        "Material",
        NodeTodo::default(),
        Schedule::default(),
    );

    let outliner = Tab::new(
        blender::OUTLINER,
        "Hierarchy",
        Hierarchy::default(),
        Hierarchy::schedule(),
    );
    let properties = Tab::new(
        blender::PROPERTIES,
        "Inspector",
        Inspector::default(),
        Inspector::schedule(),
    );

    let root = TreeNode::leaf_with(vec![node_tree, scene, material]);
    let mut split_tree = SplitTree::new(root);

    let one = TreeNode::leaf_with(vec![properties]);
    let two = TreeNode::leaf_with(vec![outliner]);
    let [_, _b] = split_tree.split(NodeIndex::root(), one, Split::Right);
    let [_, _b] = split_tree.split(_b, two, Split::Above);

    split_tree
}
