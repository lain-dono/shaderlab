use crate::app::TabInner;
use crate::context::EditorContext;
use crate::style::Style;

#[derive(Default)]
pub struct PlaceholderTab;

impl TabInner for PlaceholderTab {
    fn ui(&mut self, ui: &mut egui::Ui, style: &Style, _ctx: EditorContext) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);
    }
}
