use bevy::prelude::*;
use bevy::reflect::FromType;

pub trait Proxy {
    fn insert(self, world: &mut World, entity: Entity);
}

#[derive(Clone)]
pub struct ReflectProxy {
    resolve_and_add_func: fn(&mut World, Entity, &dyn Reflect),
}

impl ReflectProxy {
    pub fn resolve_and_add(&self, world: &mut World, entity: Entity, proxy: &dyn Reflect) {
        (self.resolve_and_add_func)(world, entity, proxy);
    }
}

impl<T: Proxy + FromWorld + Reflect> FromType<T> for ReflectProxy {
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
