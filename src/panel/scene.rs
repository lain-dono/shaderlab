use crate::app::TabInner;
use crate::context::EditorContext;
use crate::style::Style;
use bevy::prelude::*;

#[derive(Default)]
pub struct SceneTab {
    pub texture_id: Option<egui::TextureId>,
}

impl TabInner for SceneTab {
    fn ui(&mut self, ui: &mut egui::Ui, _style: &Style, _ctx: EditorContext) {
        let size = ui.available_size_before_wrap();
        if let Some(texture_id) = self.texture_id {
            ui.add(egui::widgets::Image::new(texture_id, size));
        }
    }
}

pub struct SceneRenderTarget(pub Option<Handle<Image>>);

impl SceneRenderTarget {
    pub fn insert(commands: &mut Commands, images: &mut Assets<Image>) -> Handle<Image> {
        use bevy::render::render_resource::*;

        let size = Extent3d {
            width: 1,
            height: 1,
            ..default()
        };

        // This is the texture that will be rendered to.
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..default()
        };

        // fill image.data with zeroes
        image.resize(size);

        let handle = images.add(image);

        commands.insert_resource(Self(Some(handle.clone())));

        handle
    }
}

pub fn update_scene_render_target(
    mut tree: ResMut<crate::app::SplitTree>,
    mut egui_context: ResMut<crate::shell::EguiContext>,
    scene_render_target: Res<SceneRenderTarget>,
    mut images: ResMut<Assets<Image>>,
) {
    let [ctx] = egui_context.ctx_mut([bevy::window::WindowId::primary()]);

    if let Some(handle) = scene_render_target.0.as_ref() {
        if let Some(image) = images.get_mut(handle) {
            if let Some((viewport, tab)) = tree.find_active::<SceneTab>() {
                let width = (viewport.width() * ctx.pixels_per_point()) as u32;
                let height = (viewport.height() * ctx.pixels_per_point()) as u32;
                image.resize(wgpu::Extent3d {
                    width,
                    height,
                    ..default()
                });

                tab.texture_id = Some(egui_context.add_image(handle.clone_weak()));
            }
        }
    }
}
