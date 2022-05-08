use crate::app::TabInner;
use crate::style::Style;
use bevy::prelude::*;

#[derive(Default)]
pub struct PlaceholderTab;

impl TabInner for PlaceholderTab {
    fn ui(&mut self, ui: &mut egui::Ui, style: &Style, world: &mut World) {
        let rect = ui.available_rect_before_wrap();

        /*
        let sep = rect.min.y + 25.0;

        let body_top = rect.intersect(Rect::everything_above(sep));
        ui.painter().rect_filled(body_top, 0.0, style.tab_base);

        let body = rect.intersect(Rect::everything_below(sep));
        */

        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_gray(0x28));
    }
}
