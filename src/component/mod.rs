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
            .register_type::<crate::component::ProxyMeta>()
            .register_type::<crate::component::ProxyTransform>()
            .register_type::<crate::component::ProxyPointLight>()
            .register_type::<crate::component::ProxyHandle<Mesh>>()
            .register_type::<crate::component::ProxyHandle<StandardMaterial>>();
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
