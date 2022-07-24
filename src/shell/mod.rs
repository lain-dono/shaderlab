pub mod clipboard;
pub mod convert;
pub mod node;
pub mod render;
pub mod systems;

#[cfg(test)]
mod tests;

use self::clipboard::EguiClipboard;
use self::node::{EguiNode, EguiPipeline};
use self::render::EguiTransforms;
use self::render::{extract_render_data, extract_textures, prepare_transforms, queue_bind_groups};
use self::systems::{begin_frame, init_contexts_on_startup, process_input, process_output};
use bevy::{
    app::{App, CoreStage, Plugin, StartupStage},
    asset::{AssetEvent, Assets, Handle, HandleId},
    ecs::{event::EventReader, schedule::ParallelSystemDescriptorCoercion, system::ResMut},
    render::{render_graph::RenderGraph, texture::Image, RenderApp, RenderStage},
    {input::InputSystem, log, utils::HashMap, window::WindowId},
};

/// Adds all Egui resources and render graph nodes.
pub struct EguiPlugin;

/// A resource for storing global UI settings.
#[derive(Clone, Debug, PartialEq)]
pub struct EguiSettings {
    /// Global scale factor for egui widgets (`1.0` by default).
    ///
    /// This setting can be used to force the UI to render in physical pixels regardless of DPI as follows:
    /// ```rust
    /// use bevy::prelude::*;
    /// use bevy_egui::EguiSettings;
    ///
    /// fn update_ui_scale_factor(mut settings: ResMut<EguiSettings>, windows: Res<Windows>) {
    ///     if let Some(window) = windows.get_primary() {
    ///         settings.scale_factor = 1.0 / window.scale_factor();
    ///     }
    /// }
    /// ```
    pub scale_factor: f64,
}

impl Default for EguiSettings {
    fn default() -> Self {
        Self { scale_factor: 1.0 }
    }
}

/// Is used for storing the input passed to Egui. The actual resource is `HashMap<WindowId, EguiInput>`.
///
/// It gets reset during the [`EguiSystem::ProcessInput`] system.
#[derive(Clone, Debug, Default)]
pub struct EguiRawInput {
    /// Egui's raw input.
    pub raw_input: egui::RawInput,
}

/// Is used for storing Egui shapes. The actual resource is `HashMap<WindowId, EguiShapes>`.
#[derive(Clone, Default, Debug)]
pub struct EguiRenderOutput {
    /// Pairs of rectangles and paint commands.
    ///
    /// The field gets populated during the [`EguiSystem::ProcessOutput`] system in the [`CoreStage::PostUpdate`] and reset during `EguiNode::update`.
    pub shapes: Vec<egui::epaint::ClippedShape>,

    /// The change in egui textures since last frame.
    pub textures_delta: egui::TexturesDelta,
}

/// Is used for storing Egui output. The actual resource is `HashMap<WindowId, EguiOutput>`.
#[derive(Clone, Default)]
pub struct EguiOutput {
    /// The field gets updated during the [`EguiSystem::ProcessOutput`] system in the [`CoreStage::PostUpdate`].
    pub platform_output: egui::PlatformOutput,
}

/// A resource for storing `bevy_egui` context.
#[derive(Clone)]
pub struct EguiContext {
    ctx: HashMap<WindowId, egui::Context>,
    user_textures: HashMap<HandleId, u64>,
    last_texture_id: u64,
    mouse_position: Option<(WindowId, egui::Vec2)>,
}

impl EguiContext {
    fn new() -> Self {
        Self {
            ctx: HashMap::default(),
            user_textures: Default::default(),
            last_texture_id: 0,
            mouse_position: None,
        }
    }

    /// Egui context for a specific window.
    /// If you want to display UI on a non-primary window, make sure to set up the render graph by
    /// calling [`setup_pipeline`].
    ///
    /// This function is only available when the `multi_threaded` feature is enabled.
    /// The preferable way is to use `ctx_for_window_mut` to avoid unpredictable blocking inside UI
    /// systems.
    #[cfg(feature = "multi_threaded")]
    #[must_use]
    #[track_caller]
    pub fn ctx(&self, window: WindowId) -> &egui::Context {
        self.ctx
            .get(&window)
            .unwrap_or_else(|| panic!("`EguiContext::ctx_for_window` was called for an uninitialized context (window {}), consider moving your UI system to the `CoreStage::Update` stage or run it after the `EguiSystem::BeginFrame` system (`StartupStage::Startup` or `EguiStartupSystem::InitContexts` for startup systems respectively)", window))
    }

    /// Fallible variant of [`EguiContext::ctx_for_window`]. Make sure to set up the render graph by
    /// calling [`setup_pipeline`].
    ///
    /// This function is only available when the `multi_threaded` feature is enabled.
    /// The preferable way is to use `try_ctx_for_window_mut` to avoid unpredictable blocking inside
    /// UI systems.
    #[cfg(feature = "multi_threaded")]
    #[must_use]
    pub fn try_ctx(&self, window: WindowId) -> Option<&egui::Context> {
        self.ctx.get(&window)
    }

    /// Allows to get multiple contexts at the same time. This function is useful when you want
    /// to get multiple window contexts without using the `multi_threaded` feature.
    ///
    /// # Panics
    ///
    /// Panics if the passed window ids aren't unique.
    #[must_use]
    #[track_caller]
    pub fn ctx_mut<const N: usize>(&mut self, ids: [WindowId; N]) -> [&egui::Context; N] {
        let mut unique_ids = bevy::utils::HashSet::default();
        assert!(
            ids.iter().all(move |id| unique_ids.insert(id)),
            "Window ids passed to `EguiContext::ctx_for_windows_mut` must be unique: {:?}",
            ids
        );
        ids.map(|id| self.ctx.get(&id).unwrap_or_else(|| panic!("`EguiContext::ctx_for_windows_mut` was called for an uninitialized context (window {}), consider moving your UI system to the `CoreStage::Update` stage or run it after the `EguiSystem::BeginFrame` system (`StartupStage::Startup` or `EguiStartupSystem::InitContexts` for startup systems respectively)", id)))
    }

    /// Fallible variant of [`EguiContext::ctx_for_windows_mut`]. Make sure to set up the render
    /// graph by calling [`setup_pipeline`].
    ///
    /// # Panics
    ///
    /// Panics if the passed window ids aren't unique.
    #[must_use]
    pub fn try_ctx_mut<const N: usize>(
        &mut self,
        ids: [WindowId; N],
    ) -> [Option<&egui::Context>; N] {
        let mut unique_ids = bevy::utils::HashSet::default();
        assert!(
            ids.iter().all(move |id| unique_ids.insert(id)),
            "Window ids passed to `EguiContext::ctx_for_windows_mut` must be unique: {:?}",
            ids
        );
        ids.map(|id| self.ctx.get(&id))
    }

    /// Can accept either a strong or a weak handle.
    ///
    /// You may want to pass a weak handle if you control removing texture assets in your
    /// application manually and you don't want to bother with cleaning up textures in Egui.
    ///
    /// You'll want to pass a strong handle if a texture is used only in Egui and there's no
    /// handle copies stored anywhere else.
    pub fn add_image(&mut self, image: Handle<Image>) -> egui::TextureId {
        let id = *self.user_textures.entry(image.id).or_insert_with(|| {
            let id = self.last_texture_id;
            log::debug!("Add a new image (id: {}, handle: {:?})", id, image);
            self.last_texture_id += 1;
            id
        });
        egui::TextureId::User(id)
    }

    /// Removes the image handle and an Egui texture id associated with it.
    pub fn remove_image(&mut self, image: &Handle<Image>) -> Option<egui::TextureId> {
        let id = self.user_textures.remove(&image.id);
        log::debug!("Remove image (id: {:?}, handle: {:?})", id, image);
        id.map(egui::TextureId::User)
    }

    /// Returns an associated Egui texture id.
    #[must_use]
    pub fn image_id(&self, image: &Handle<Image>) -> Option<egui::TextureId> {
        self.user_textures
            .get(&image.id)
            .map(|&id| egui::TextureId::User(id))
    }
}

#[doc(hidden)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct WindowSize {
    physical_width: u32,
    physical_height: u32,
    scale_factor: f32,
}

impl WindowSize {
    fn new(physical_width: u32, physical_height: u32, scale_factor: f32) -> Self {
        Self {
            physical_width,
            physical_height,
            scale_factor,
        }
    }

    #[inline]
    fn scaled_width(&self) -> f32 {
        self.physical_width as f32 / self.scale_factor
    }

    #[inline]
    fn scaled_height(&self) -> f32 {
        self.physical_height as f32 / self.scale_factor
    }
}

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        use CoreStage::{Last, PostUpdate, PreUpdate};
        use RenderStage::{Extract, Prepare, Queue};
        use StartupStage::PreStartup;

        let world = &mut app.world;
        world.insert_resource(EguiSettings::default());
        world.insert_resource(HashMap::<WindowId, EguiRawInput>::default());
        world.insert_resource(HashMap::<WindowId, EguiOutput>::default());
        world.insert_resource(HashMap::<WindowId, WindowSize>::default());
        world.insert_resource(HashMap::<WindowId, EguiRenderOutput>::default());
        world.insert_resource(EguiManagedTextures::default());
        world.insert_resource(EguiClipboard::default());
        world.insert_resource(EguiContext::new());

        app.add_startup_system_to_stage(PreStartup, init_contexts_on_startup);
        app.add_system_to_stage(PreUpdate, process_input.after(InputSystem));
        app.add_system_to_stage(PreUpdate, begin_frame.after(process_input));
        app.add_system_to_stage(PostUpdate, process_output);
        app.add_system_to_stage(PostUpdate, update_textures.after(process_output));
        app.add_system_to_stage(Last, free_textures);

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<EguiPipeline>()
                .init_resource::<EguiTransforms>()
                .add_system_to_stage(Extract, extract_render_data)
                .add_system_to_stage(Extract, extract_textures)
                .add_system_to_stage(Prepare, prepare_transforms)
                .add_system_to_stage(Queue, queue_bind_groups);
        }
    }
}

#[derive(Default)]
pub struct EguiManagedTextures(HashMap<(WindowId, u64), EguiManagedTexture>);

pub struct EguiManagedTexture {
    handle: Handle<Image>,
    /// Stored in full so we can do partial updates (which bevy doesn't support).
    color_image: egui::ColorImage,
}

fn update_textures(
    mut render_output: ResMut<HashMap<WindowId, EguiRenderOutput>>,
    mut managed_textures: ResMut<EguiManagedTextures>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    for (&window_id, render_output) in render_output.iter_mut() {
        let set_textures = std::mem::take(&mut render_output.textures_delta.set);

        for (texture_id, image_delta) in set_textures {
            let color_image = node::as_color_image(image_delta.image);

            let texture_id = match texture_id {
                egui::TextureId::Managed(texture_id) => texture_id,
                egui::TextureId::User(_) => continue,
            };

            if let Some(pos) = image_delta.pos {
                // Partial update.
                if let Some(managed_texture) = managed_textures.0.get_mut(&(window_id, texture_id))
                {
                    // TODO: when bevy supports it, only update the part of the texture that changes.
                    update_image_rect(&mut managed_texture.color_image, pos, &color_image);
                    let image = node::color_image_as_bevy_image(&managed_texture.color_image);
                    managed_texture.handle = image_assets.add(image);
                } else {
                    log::warn!("Partial update of a missing texture (id: {:?})", texture_id);
                }
            } else {
                // Full update.
                let image = node::color_image_as_bevy_image(&color_image);
                let handle = image_assets.add(image);
                managed_textures.0.insert(
                    (window_id, texture_id),
                    EguiManagedTexture {
                        handle,
                        color_image,
                    },
                );
            }
        }
    }

    fn update_image_rect(dest: &mut egui::ColorImage, [x, y]: [usize; 2], src: &egui::ColorImage) {
        for sy in 0..src.height() {
            for sx in 0..src.width() {
                dest[(x + sx, y + sy)] = src[(sx, sy)];
            }
        }
    }
}

fn free_textures(
    mut context: ResMut<EguiContext>,
    mut render_output: ResMut<HashMap<WindowId, EguiRenderOutput>>,
    mut managed_textures: ResMut<EguiManagedTextures>,
    mut image_assets: ResMut<Assets<Image>>,
    mut image_events: EventReader<AssetEvent<Image>>,
) {
    for (&window_id, render_ouput) in render_output.iter_mut() {
        let free_textures = std::mem::take(&mut render_ouput.textures_delta.free);
        for texture_id in free_textures {
            if let egui::TextureId::Managed(texture_id) = texture_id {
                let managed_texture = managed_textures.0.remove(&(window_id, texture_id));
                if let Some(managed_texture) = managed_texture {
                    image_assets.remove(managed_texture.handle);
                }
            }
        }
    }

    for image_event in image_events.iter() {
        if let AssetEvent::Removed { handle } = image_event {
            context.remove_image(handle);
        }
    }
}

/// Set up egui render pipeline.
///
/// The pipeline for the primary window will already be set up by the [`EguiPlugin`],
/// so you'll only need to manually call this if you want to use multiple windows.
pub fn setup_pipeline(graph: &mut RenderGraph, window_id: WindowId, pass: &'static str) {
    graph.add_node(pass, EguiNode::new(window_id));

    let main_driver = bevy::render::main_graph::node::CAMERA_DRIVER;
    graph.add_node_edge(main_driver, pass).unwrap();

    let _ = graph.add_node_edge("ui_pass_driver", pass);
}
