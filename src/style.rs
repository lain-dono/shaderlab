use crate::util::map_to_pixel;
use egui::{Color32, CursorIcon, Rect, Rounding, Sense, Ui};

#[derive(Clone)]
pub struct Style {
    pub separator_size: f32,
    pub separator_extra: f32,

    pub app_bg: Color32,

    pub top_base: Color32,
    pub top_text: Color32,
    pub top_active: Color32,
    pub top_disable: Color32,

    pub tab_rounding: Rounding,
    pub tab_bar: Color32,
    pub tab_base: Color32,
    pub tab_text: Color32,
    pub tab_active: Color32,
    pub tab_outline: Color32,

    pub selection: Color32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            separator_size: 2.0,
            separator_extra: 80.0,

            app_bg: Color32::from_gray(0x14),

            top_base: Color32::from_gray(0x2d),
            top_text: Color32::from_gray(0x9d),
            top_active: Color32::from_gray(0x55),
            top_disable: Color32::from_gray(0x20),

            tab_rounding: Rounding {
                ne: 2.0,
                nw: 2.0,
                se: 0.0,
                sw: 0.0,
            },
            tab_bar: Color32::from_gray(0x20),
            tab_base: Color32::from_gray(0x30),
            tab_text: Color32::from_gray(0x9d),
            tab_active: Color32::from_gray(0x40),
            tab_outline: Color32::from_gray(0x1c),

            selection: Color32::from_rgba_unmultiplied(61, 133, 224, 30),
        }
    }
}

impl Style {
    pub fn hsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = midpoint - self.separator_size * 0.5;
        separator.max.x = midpoint + self.separator_size * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::ResizeHorizontal);

        {
            let delta = response.drag_delta().x;
            let range = rect.max.x - rect.min.x;
            let min = (self.separator_extra / range).min(1.0);
            let max = 1.0 - min;
            let (min, max) = (min.min(max), max.max(min));
            *fraction = (*fraction + delta / range).clamp(min, max);
        }

        let midpoint = rect.min.x + rect.width() * *fraction;
        separator.min.x = map_to_pixel(
            midpoint - self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.x = map_to_pixel(
            midpoint + self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            rect.intersect(Rect::everything_right_of(separator.max.x)),
            separator,
            rect.intersect(Rect::everything_left_of(separator.min.x)),
        )
    }

    pub fn vsplit(&self, ui: &mut Ui, fraction: &mut f32, rect: Rect) -> (Rect, Rect, Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();

        let mut separator = rect;

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = midpoint - self.separator_size * 0.5;
        separator.max.y = midpoint + self.separator_size * 0.5;

        let response = ui
            .allocate_rect(separator, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::ResizeVertical);

        {
            let delta = response.drag_delta().y;
            let range = rect.max.y - rect.min.y;

            let min = (self.separator_extra / range).min(1.0);
            let max = 1.0 - min;
            let (min, max) = (min.min(max), max.max(min));
            *fraction = (*fraction + delta / range).clamp(min, max);
        }

        let midpoint = rect.min.y + rect.height() * *fraction;
        separator.min.y = map_to_pixel(
            midpoint - self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );
        separator.max.y = map_to_pixel(
            midpoint + self.separator_size * 0.5,
            pixels_per_point,
            f32::round,
        );

        (
            rect.intersect(Rect::everything_above(separator.min.y)),
            separator,
            rect.intersect(Rect::everything_below(separator.max.y)),
        )
    }
}
