use crate::component::{ComponentEditor, Proxy, ReflectComponentEditor, ReflectProxy};
use bevy::prelude::*;
use bevy::reflect::FromReflect;
use std::borrow::Cow;

#[derive(Component, Reflect, FromReflect)]
#[reflect(Component, Proxy, ComponentEditor)]
pub struct ProxyTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl ProxyTransform {
    pub fn to_local(&self) -> Transform {
            Transform {
                translation: self.translation,
                rotation: self.rotation,
                scale: self.scale,
            }
    }

    pub fn to_global(&self) -> GlobalTransform {
        GlobalTransform {
            translation: self.translation,
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}

impl Default for ProxyTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Proxy for ProxyTransform {
    fn insert(self, world: &mut World, entity: Entity) {
        world.entity_mut(entity).insert_bundle((
            self.to_local(),
            GlobalTransform::default(),
        ));
    }
}

impl ComponentEditor for ProxyTransform {
    fn desc() -> (char, Cow<'static, str>) {
        (crate::icon::ORIENTATION_LOCAL, "Transform".into())
    }
}
