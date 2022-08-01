use crate::ui::EditorTab;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;

#[derive(Default, Component)]
pub struct PlaceholderTab;

impl EditorTab for PlaceholderTab {
    type Param = Res<'static, crate::ui::Style>;

    fn ui<'w>(
        &mut self,
        ui: &mut egui::Ui,
        _entity: Entity,
        style: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);
    }
}
