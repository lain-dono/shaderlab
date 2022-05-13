mod proxy;
mod scene;

pub use self::proxy::{Proxy, ProxyHandle, ProxyMeta, ProxyTransform, ReflectProxyComponent};
pub use self::scene::{
    ReflectEntity, ReflectScene, ReflectSceneLoader, ReflectSceneSpawnError, ReflectSceneSpawner,
    ScenePlugin,
};

pub struct EditroAssetPlugin;

impl bevy::app::Plugin for EditroAssetPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        use bevy::prelude::*;

        app.add_plugin(ScenePlugin)
            .register_type::<crate::asset::ProxyMeta>()
            .register_type::<crate::asset::ProxyTransform>()
            .register_type::<crate::asset::ProxyHandle<Mesh>>()
            .register_type::<crate::asset::ProxyHandle<StandardMaterial>>();
    }
}
