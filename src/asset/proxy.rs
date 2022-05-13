use bevy::asset::{Asset, AssetPath};
use bevy::prelude::*;
use bevy::reflect::{FromReflect, FromType};
use std::borrow::Cow;
use std::marker::PhantomData;

pub trait Proxy {
    fn insert(self, world: &mut World, entity: Entity);
}

#[derive(Clone)]
pub struct ReflectProxyComponent {
    resolve_and_add_func: fn(&mut World, Entity, &dyn Reflect),
}

impl ReflectProxyComponent {
    pub fn resolve_and_add(&self, world: &mut World, entity: Entity, proxy: &dyn Reflect) {
        (self.resolve_and_add_func)(world, entity, proxy);
    }
}

impl<T: Proxy + FromWorld + Reflect> FromType<T> for ReflectProxyComponent {
    fn from_type() -> Self {
        Self {
            resolve_and_add_func: |world, entity, reflected_proxy| {
                let mut proxy = T::from_world(world);
                proxy.apply(reflected_proxy);
                proxy.insert(world, entity);
            },
        }
    }
}

// ---------------------
// TODO: remove Component derive

use crate::panel::inspector::{ComponentEditor, ReflectComponentEditor};

#[derive(Component, Reflect, FromReflect)]
#[reflect(Component, ProxyComponent, ComponentEditor)]
pub struct ProxyHandle<T: Asset> {
    pub path: String,

    #[reflect(ignore)]
    marker: PhantomData<fn() -> T>,
}

impl<T: Asset> ProxyHandle<T> {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            marker: PhantomData,
        }
    }
}

impl<T: Asset> Default for ProxyHandle<T> {
    fn default() -> Self {
        Self {
            path: Default::default(),
            marker: PhantomData,
        }
    }
}

impl<T: Asset> ProxyHandle<T> {
    fn resolve(self, world: &mut World) -> Option<Handle<T>> {
        let asset_server = world.get_resource::<AssetServer>()?;
        Some(asset_server.load(AssetPath::from(&self.path)))
    }
}

impl<T: Asset> Proxy for ProxyHandle<T> {
    fn insert(self, world: &mut World, entity: Entity) {
        if let Some(component) = self.resolve(world) {
            world.entity_mut(entity).insert(component);
        }
    }
}

#[derive(Component, Reflect, FromReflect)]
#[reflect(Component, ProxyComponent, ComponentEditor)]
pub struct ProxyTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
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
            Transform {
                translation: self.translation,
                rotation: self.rotation,
                scale: self.scale,
            },
            GlobalTransform::default(),
        ));
    }
}

#[derive(Component, Reflect, FromReflect)]
#[reflect(Component, ProxyComponent, ComponentEditor)]
pub struct ProxyMeta {
    pub icon: u32,
    pub name: Cow<'static, str>,
    pub is_visible: bool,
}

impl ProxyMeta {
    pub fn new(icon: char, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            icon: icon as u32,
            name: name.into(),
            is_visible: true,
        }
    }
}

impl Default for ProxyMeta {
    fn default() -> Self {
        Self::new('?', "Entity")
    }
}

impl Proxy for ProxyMeta {
    fn insert(self, world: &mut World, entity: Entity) {
        world.entity_mut(entity).insert_bundle((
            Name::new(self.name),
            Visibility {
                is_visible: self.is_visible,
            },
            ComputedVisibility {
                is_visible: self.is_visible,
            },
        ));
    }
}
