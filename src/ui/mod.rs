pub mod app;
pub mod field;
pub mod icon;
pub mod placeholder;
pub mod shell;
pub mod style;
pub mod tabs;

pub use self::app::{EditorPanel, EditorTab};
pub use self::placeholder::PlaceholderTab;
pub use self::style::Style;
pub use self::tabs::{NodeIndex, Split, SplitTree, Tab, TreeNode};

use bevy::ecs::system::StaticSystemParam;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::{render_graph::RenderGraph, RenderApp};
use bevy::window::WindowId;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum EditorStage {
    Root,
    Tabs,
    Finish,
}

pub struct EditorUiPlugin;

impl bevy::app::Plugin for EditorUiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        use self::app::*;
        use bevy::prelude::*;

        app.add_plugin(self::shell::EguiPlugin)
            .init_resource::<self::style::Style>()
            .init_resource::<SharedData>()
            .add_startup_system(setup_icon_font)
            .add_stage_after(
                CoreStage::PreUpdate,
                EditorStage::Root,
                SystemStage::parallel(),
            )
            .add_stage_after(
                CoreStage::Update,
                EditorStage::Tabs,
                SystemStage::parallel(),
            )
            .add_stage_before(
                CoreStage::PostUpdate,
                EditorStage::Finish,
                SystemStage::parallel(),
            )
            .add_system_to_stage(EditorStage::Root, ui_root)
            .add_system_to_stage(EditorStage::Root, update_panel_render_target.after(ui_root))
            .add_system_to_stage(EditorStage::Finish, ui_tabs)
            .add_system_to_stage(EditorStage::Finish, ui_finish.after(ui_tabs))
            .add_editor_tab::<PlaceholderTab>();

        {
            let render_app = app.sub_app_mut(RenderApp);
            let mut graph = render_app.world.resource_mut::<RenderGraph>();

            // add egui nodes
            crate::ui::shell::setup_pipeline(&mut graph, WindowId::primary(), "ui_root");
        }
    }
}

fn setup_icon_font(mut context: ResMut<self::shell::EguiContext>) {
    let font = egui::FontData::from_static(include_bytes!("icon.ttf"));

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("blender".to_owned(), font);
    fonts.families.insert(
        egui::FontFamily::Name("blender".into()),
        vec!["Hack".to_owned(), "blender".into()],
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .push("blender".to_owned());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("blender".to_owned());

    let [ctx] = context.ctx_mut([WindowId::primary()]);
    ctx.set_fonts(fonts);
}

pub trait AddEditorTab {
    fn add_editor_tab<T: EditorTab + 'static>(&mut self) -> &mut Self;
}

impl AddEditorTab for bevy::app::App {
    fn add_editor_tab<T: EditorTab + 'static>(&mut self) -> &mut Self {
        fn system<T: EditorTab>(
            mut context: ResMut<crate::ui::shell::EguiContext>,
            mut panel_view: Query<(Entity, &EditorPanel, &mut T)>,
            query: StaticSystemParam<T::Param>,
        ) {
            let [ctx] = context.ctx_mut([bevy::window::WindowId::primary()]);
            let mut query = query.into_inner();
            for (entity, viewport, mut tab) in panel_view.iter_mut() {
                if let Some(viewport) = viewport.viewport {
                    let mut ui = egui::Ui::new(
                        ctx.clone(),
                        egui::LayerId::background(),
                        egui::Id::new(entity),
                        viewport,
                        viewport,
                    );
                    tab.ui(&mut ui, entity, &mut query);
                }
            }
        }

        self.add_system_to_stage(EditorStage::Tabs, system::<T>);
        self
    }
}

#[derive(Default, Component)]
pub struct PanelRenderTarget {
    pub texture_id: Option<egui::TextureId>,
}

impl PanelRenderTarget {
    pub fn create_render_target(images: &mut Assets<Image>) -> RenderTarget {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let mut image = Image {
            texture_descriptor: wgpu::TextureDescriptor {
                label: None,
                size,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
            },
            ..default()
        };

        // fill image.data with zeroes
        image.resize(size);

        RenderTarget::Image(images.add(image))
    }
}

pub fn update_panel_render_target(
    mut context: ResMut<self::shell::EguiContext>,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<(&mut PanelRenderTarget, &EditorPanel, &Camera)>,
) {
    let [ctx] = context.ctx_mut([bevy::window::WindowId::primary()]);
    let ppi = ctx.pixels_per_point();

    for (mut panel, tab, camera) in query.iter_mut() {
        let handle = if let RenderTarget::Image(handle) = &camera.target {
            handle
        } else {
            continue;
        };

        if let Some(image) = images.get_mut(handle) {
            if let Some(viewport) = tab.viewport {
                let width = (viewport.width() * ppi) as u32;
                let height = (viewport.height() * ppi) as u32;

                panel.texture_id = Some(context.add_image(handle.clone_weak()));

                image.resize(wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    depth_or_array_layers: 1,
                });
            }
        }
    }
}
