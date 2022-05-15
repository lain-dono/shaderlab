use crate::component::{ComponentEditor, Proxy, ReflectComponentEditor, ReflectProxy};
use bevy::prelude::*;
use bevy::reflect::FromReflect;
use std::borrow::Cow;

#[derive(Component, Reflect, FromReflect)]
#[reflect(Component, Proxy, ComponentEditor)]
pub struct ProxyPointLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub shadows_enabled: bool,
    pub shadow_depth_bias: f32,
    pub shadow_normal_bias: f32,
}

impl Default for ProxyPointLight {
    fn default() -> Self {
        Self {
            color: Color::rgb(1.0, 1.0, 1.0),
            /// Luminous power in lumens
            intensity: 800.0, // Roughly a 60W non-halogen incandescent bulb
            range: 20.0,
            radius: 0.0,
            shadows_enabled: false,
            shadow_depth_bias: PointLight::DEFAULT_SHADOW_DEPTH_BIAS,
            shadow_normal_bias: PointLight::DEFAULT_SHADOW_NORMAL_BIAS,
        }
    }
}

impl Proxy for ProxyPointLight {
    fn insert(self, world: &mut World, entity: Entity) {
        world.entity_mut(entity).insert_bundle((
            PointLight {
                color: self.color,
                intensity: self.intensity,
                range: self.range,
                radius: self.radius,
                shadows_enabled: self.shadows_enabled,
                shadow_depth_bias: self.shadow_depth_bias,
                shadow_normal_bias: self.shadow_normal_bias,
            },
            bevy::pbr::CubemapVisibleEntities::default(),
            bevy::render::primitives::CubemapFrusta::default(),
        ));
    }
}

impl ComponentEditor for ProxyPointLight {
    fn desc() -> (char, Cow<'static, str>) {
        (crate::icon::LIGHT_POINT, "PointLight".into())
    }
}
