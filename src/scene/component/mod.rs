pub mod editor;
pub mod proxy;

mod handle;
mod light;
mod meta;
mod transform;

pub use self::editor::{reflect_component_editor, ComponentEditor, ReflectComponentEditor};
pub use self::handle::ProxyHandle;
pub use self::light::ProxyPointLight;
pub use self::meta::ProxyMeta;
pub use self::proxy::{Proxy, ReflectProxy};
pub use self::transform::ProxyTransform;

pub struct EditorPlugin;

impl bevy::app::Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        use bevy::prelude::*;

        app.add_plugin(crate::scene::ScenePlugin)
            .register_type::<ProxyMeta>()
            .register_type::<ProxyTransform>()
            .register_type::<ProxyPointLight>()
            .register_type::<ProxyHandle<Mesh>>()
            .register_type::<ProxyHandle<StandardMaterial>>();
    }
}

impl ComponentEditor for bevy::prelude::Parent {
    fn skip() -> bool {
        true
    }
}

impl ComponentEditor for bevy::prelude::Children {
    fn skip() -> bool {
        true
    }
}
