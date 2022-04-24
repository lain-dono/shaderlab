use bevy::prelude::*;
use std::borrow::Cow;

pub struct Global {
    pub world: World,
    pub selected: Option<Entity>,
}

impl Default for Global {
    fn default() -> Self {
        // TODO: derive clean version

        Self {
            world: exampe_world(),
            selected: None,
        }
    }
}

#[derive(Bundle, Default)]
pub struct EditorBundle {
    pub icon: Icon,
    pub name: Name,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,

    pub state: EditorEntityState,
}

impl EditorBundle {
    pub fn new(icon: char, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            icon: Icon { icon },
            name: Name::new(name),
            state: EditorEntityState { is_open: true },
            ..Default::default()
        }
    }
}

#[derive(Component, Default)]
pub struct EditorEntityState {
    pub is_open: bool,
}

#[derive(Component, Default)]
pub struct Icon {
    pub icon: char,
}

fn exampe_world() -> World {
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

    world
}
