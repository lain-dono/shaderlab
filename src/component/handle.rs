use crate::component::{ComponentEditor, Proxy, ReflectComponentEditor, ReflectProxy};
use crate::style::Style;
use bevy::asset::{Asset, AssetPath};
use bevy::prelude::*;
use bevy::reflect::{FromReflect, GetPath};
use std::borrow::Cow;
use std::marker::PhantomData;

#[derive(Component, Reflect, FromReflect)]
#[reflect(Component, Proxy, ComponentEditor)]
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

impl<T: bevy::asset::Asset> ComponentEditor for ProxyHandle<T> {
    fn desc() -> (char, Cow<'static, str>) {
        let type_name = std::any::type_name::<T>();
        if type_name == std::any::type_name::<bevy::prelude::Mesh>() {
            (crate::icon::MESH_DATA, "Mesh".into())
        } else if type_name == std::any::type_name::<bevy::prelude::StandardMaterial>() {
            (crate::icon::MATERIAL_DATA, "Material".into())
        } else {
            (' ', format!("Handle<{}>", type_name).into())
        }
    }

    fn ui(ui: &mut egui::Ui, style: &Style, reflect: &mut dyn Reflect) {
        let (icon, name) = Self::desc();
        //crate::component::reflect_component_editor(ui, style, reflect, icon, &name);
        crate::component::editor::component_header(None, ui, style, icon, &name, |ui| {
            let field = reflect.path_mut("path").unwrap();
            crate::field::reflect(ui, field);
        });
    }
}
