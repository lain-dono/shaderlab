use super::{
    node::EguiPipeline, EguiContext, EguiManagedTextures, EguiRenderOutput, EguiSettings,
    WindowSize,
};
use bevy::prelude::*;
use bevy::render::render_resource::{DynamicUniformBuffer, ShaderType};
use bevy::render::Extract;
use bevy::render::{
    render_resource::{BindGroup, BufferId},
    renderer::{RenderDevice, RenderQueue},
    {render_asset::RenderAssets, texture::Image},
};
use bevy::{asset::HandleId, utils::HashMap, window::WindowId};
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource};

pub struct ExtractedRenderOutput(pub HashMap<WindowId, EguiRenderOutput>);
pub struct ExtractedWindowSizes(pub HashMap<WindowId, WindowSize>);
pub struct ExtractedEguiContext(pub HashMap<WindowId, egui::Context>);
pub struct ExtractedEguiSettings(pub EguiSettings);

pub struct EguiTextureBindGroups {
    pub bind_groups: HashMap<EguiTexture, BindGroup>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EguiTexture {
    /// Textures allocated via egui.
    Managed(WindowId, u64),
    /// Textures allocated via bevy.
    User(u64),
}

pub struct ExtractedEguiTextures {
    pub egui_textures: HashMap<(WindowId, u64), Handle<Image>>,
    pub user_textures: HashMap<Handle<Image>, u64>,
}

impl ExtractedEguiTextures {
    pub fn handles(&self) -> impl Iterator<Item = (EguiTexture, HandleId)> + '_ {
        self.egui_textures
            .iter()
            .map(|(&(window_id, texture_id), handle)| {
                (EguiTexture::Managed(window_id, texture_id), handle.id)
            })
            .chain(
                self.user_textures
                    .iter()
                    .map(|(handle, &id)| (EguiTexture::User(id), handle.id)),
            )
    }
}

#[derive(Default)]
pub struct EguiTransforms {
    pub buffer: DynamicUniformBuffer<EguiTransform>,
    pub offsets: HashMap<WindowId, u32>,
    pub bind_group: Option<(BufferId, BindGroup)>,
}

#[derive(ShaderType, Default)]
pub struct EguiTransform {
    scale: Vec2,
    translation: Vec2,
}

impl EguiTransform {
    fn new(window_size: WindowSize, scale_factor: f32) -> Self {
        let x = 2.0 / (window_size.scaled_width() / scale_factor);
        let y = -2.0 / (window_size.scaled_height() / scale_factor);
        Self {
            scale: Vec2::new(x, y),
            translation: Vec2::new(-1.0, 1.0),
        }
    }
}

pub fn extract_render_data(
    mut commands: Commands,
    render_output: Extract<Res<HashMap<WindowId, EguiRenderOutput>>>,
    sizes: Extract<Res<HashMap<WindowId, WindowSize>>>,
    settings: Extract<Res<EguiSettings>>,
    context: Extract<Res<EguiContext>>,
) {
    commands.insert_resource(ExtractedRenderOutput(render_output.clone()));
    commands.insert_resource(ExtractedEguiSettings(settings.clone()));
    commands.insert_resource(ExtractedEguiContext(context.ctx.clone()));
    commands.insert_resource(ExtractedWindowSizes(sizes.clone()));
}

pub fn extract_textures(
    mut commands: Commands,
    context: Extract<Res<EguiContext>>,
    textures: Extract<Res<EguiManagedTextures>>,
) {
    commands.insert_resource(ExtractedEguiTextures {
        egui_textures: textures
            .0
            .iter()
            .map(|(&(window_id, texture_id), managed_texture)| {
                ((window_id, texture_id), managed_texture.handle.clone())
            })
            .collect(),
        user_textures: context.user_textures.clone(),
    });
}

pub fn prepare_transforms(
    mut transforms: ResMut<EguiTransforms>,
    window_sizes: Res<ExtractedWindowSizes>,
    settings: Res<ExtractedEguiSettings>,
    pipeline: Res<EguiPipeline>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    transforms.buffer.clear();
    transforms.offsets.clear();

    for (window, size) in &window_sizes.0 {
        let offset = transforms
            .buffer
            .push(EguiTransform::new(*size, settings.0.scale_factor as f32));
        transforms.offsets.insert(*window, offset);
    }

    transforms.buffer.write_buffer(&device, &queue);

    if let Some(buffer) = transforms.buffer.buffer() {
        match transforms.bind_group {
            Some((id, _)) if buffer.id() == id => {}
            _ => {
                let transform_bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("egui transform bind group"),
                    layout: &pipeline.transform_bind_group_layout,
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: transforms.buffer.binding().unwrap(),
                    }],
                });
                transforms.bind_group = Some((buffer.id(), transform_bind_group));
            }
        };
    }
}

pub fn queue_bind_groups(
    mut commands: Commands,
    textures: Res<ExtractedEguiTextures>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<Image>>,
    pipeline: Res<EguiPipeline>,
) {
    let bind_groups = textures
        .handles()
        .filter_map(|(texture, handle_id)| {
            let gpu_image = gpu_images.get(&Handle::weak(handle_id))?;
            let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &pipeline.texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&gpu_image.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&gpu_image.sampler),
                    },
                ],
            });
            Some((texture, bind_group))
        })
        .collect();

    commands.insert_resource(EguiTextureBindGroups { bind_groups })
}
