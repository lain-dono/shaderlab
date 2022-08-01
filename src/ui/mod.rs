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

pub fn fonts_with_blender() -> egui::FontDefinitions {
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

    fonts
}

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
            .init_resource::<SharedData>()
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
            .add_system_to_stage(EditorStage::Finish, ui_tabs)
            .add_system_to_stage(EditorStage::Finish, ui_finish.after(ui_tabs))
            .add_editor_tab::<PlaceholderTab>();
    }
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
