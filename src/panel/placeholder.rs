use crate::app::EditorPanel;
use crate::style::Style;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::prelude::*;
use bevy::window::WindowId;

#[derive(Default, Component)]
pub struct PlaceholderTab;

impl PlaceholderTab {
    pub fn system(
        mut context: ResMut<crate::shell::EguiContext>,
        style: Res<Style>,
        query: Query<(Entity, &EditorPanel), With<PlaceholderTab>>,
    ) {
        let [ctx] = context.ctx_mut([WindowId::primary()]);
        for (entity, viewport) in query.iter() {
            if let Some(viewport) = viewport.viewport {
                let id = egui::Id::new("Placeholder").with(entity);
                let ui = egui::Ui::new(
                    ctx.clone(),
                    egui::LayerId::background(),
                    id,
                    viewport,
                    viewport,
                );

                let rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(rect, 0.0, style.panel);
            }
        }
    }
}

impl crate::app::EditorTab for PlaceholderTab {
    type Param = SRes<Style>;

    fn ui<'w>(
        &mut self,
        ui: &mut egui::Ui,
        _entity: Entity,
        style: &mut bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
    ) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);
    }
}
