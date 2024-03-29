use super::{Animation, BoneTimeline, Controller, Interpolation, Keyframe, PlayControl, PlayState};
use crate::ui::{icon, EditorTab, Style};
use bevy::ecs::entity::Entity;
use bevy::ecs::system::lifetimeless::{SRes, SResMut};
use bevy::ecs::system::SystemParamItem;
use egui::*;

const LINE_HEIGHT: f32 = 20.0;
const HEADER_HEIGHT: f32 = 24.0;

const LEFT_PADDING: f32 = 200.0;

const BONE_BG_COLOR: Color32 = Color32::from_gray(0x2F);
const BONE_BG_COLOR_HOVER: Color32 = Color32::from_gray(0x3F);
const KEY_DIVIDER_COLOR: Color32 = Color32::from_gray(0x23);
const KEY_DIVIDER_CURRENT_COLOR: Color32 = Color32::from_rgb(61, 133, 224);

const KEYFRAME_COLOR: Color32 = Color32::from_gray(0xAA);
const KEYFRAME_COLOR_HOVER: Color32 = Color32::from_gray(0xFF);

const CURVE_COLOR: Color32 = Color32::from_gray(0xFF);
const CURVE_COLOR_FACTOR: f32 = 0.125;

const KEYFRAME_SIZE: f32 = 16.0;

const CONTROL_SIZE: f32 = 20.0;

const TRANSLATE_COLOR: Color32 = Color32::from_rgb(0x2E, 0xCC, 0x40);
const ROTATE_COLOR: Color32 = Color32::from_rgb(0xFF, 0x41, 0x36);
const SCALE_COLOR: Color32 = Color32::from_rgb(0xFF, 0x85, 0x1B);
const SHEAR_COLOR: Color32 = Color32::from_rgb(0xFF, 0x85, 0x1B);

const ROW_WIDTH: f32 = 18.0;

#[derive(Default, bevy::prelude::Component)]
pub struct TimelinePanel;

impl EditorTab for TimelinePanel {
    type Param = (SRes<Style>, SResMut<Animation>, SResMut<Controller>);

    fn ui<'w>(
        &mut self,
        ui: &mut egui::Ui,
        _entity: Entity,
        (style, animation, controller): &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, BONE_BG_COLOR);

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            style.set_theme_visuals(ui);

            // title bar
            ui.scope(|ui| {
                let max_width = ui.available_size_before_wrap().x;
                let size = egui::vec2(max_width, HEADER_HEIGHT);

                let bg_idx = ui.painter().add(Shape::Noop);

                let response = ui.allocate_ui(size, |ui| {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
                        ui.add_space(2.0);

                        let prev_style = (*ui.ctx().style()).clone();
                        let mut style = prev_style.clone();

                        use egui::FontFamily::Proportional;
                        use egui::TextStyle::*;
                        style.text_styles = [
                            (Body, FontId::new(CONTROL_SIZE, Proportional)),
                            (Button, FontId::new(CONTROL_SIZE, Proportional)),
                        ]
                        .into();
                        ui.ctx().set_style(style);

                        if let Some(control) = play_control(ui, controller.state) {
                            controller.action(control, animation.duration());
                        }

                        ui.ctx().set_style(prev_style);
                    });
                    ui.add_space(2.0);
                });

                let rect = response.response.rect;
                let rect = Rect::from_min_size(rect.min, vec2(max_width, rect.height()));
                let shape = Shape::rect_filled(rect, 0.0, style.panel);
                ui.painter().set(bg_idx, shape);

                let left_split = rect.min.x + LEFT_PADDING;
                let rect = rect.intersect(Rect::everything_right_of(left_split));

                let id = Id::new("timeline_current_time");
                let outer_response = ui.interact(rect, id, Sense::click_and_drag());
                let outer_response = outer_response.on_hover_cursor(egui::CursorIcon::PointingHand);

                let is_down = outer_response.is_pointer_button_down_on();

                let count = (rect.width() / ROW_WIDTH) as u32;
                let ppi = ui.ctx().pixels_per_point();

                let painter = ui.painter();
                for i in 0..=count {
                    let x = rect.min.x + ROW_WIDTH * i as f32;
                    let x = crate::util::map_to_pixel(x, ppi, f32::floor);

                    let height = rect.height();
                    let center = pos2(x, rect.min.y + height / 2.0);
                    let size = vec2(ROW_WIDTH, height);
                    let item_rect = Rect::from_center_size(center, size);
                    let bg_idx = ui.painter().add(Shape::Noop);

                    {
                        let id = Id::new("timeline_current_time").with(i);
                        let response = ui.interact(item_rect, id, Sense::hover());

                        if is_down && response.hovered() {
                            controller.current_time = i;
                        }
                    }

                    let pos = item_rect.center();
                    let font_id = egui::FontId::monospace(10.0);
                    painter.text(pos, Align2::CENTER_CENTER, i, font_id, style.input_text);

                    let bg_rect = item_rect.shrink2(vec2(1.0, 4.0));
                    let px = ppi.recip();
                    if i == controller.current_time {
                        ui.painter().set(
                            bg_idx,
                            Shape::rect_filled(bg_rect, 2.0, KEY_DIVIDER_CURRENT_COLOR),
                        );

                        ui.painter().line_segment(
                            [item_rect.center_bottom(), bg_rect.center_bottom()],
                            (px, KEY_DIVIDER_CURRENT_COLOR),
                        );
                    } else {
                        ui.painter().line_segment(
                            [item_rect.center_bottom(), bg_rect.center_bottom()],
                            (px, KEY_DIVIDER_COLOR),
                        );
                    }
                }
            });

            style.for_scrollbar(ui);

            let scroll = ScrollArea::vertical()
                .auto_shrink([false; 2])
                .id_source("Timeline ScrollArea");

            scroll.show(ui, |ui| {
                style.scrollarea(ui);

                for bone in &mut animation.bones {
                    bone.draw(ui, style, controller.current_time);
                }
            });
        });
    }
}

fn play_control(ui: &mut Ui, state: PlayState) -> Option<PlayControl> {
    if ui.button(icon::TRIA_LEFT_BAR.to_string()).clicked() {
        return Some(PlayControl::First);
    }
    if ui.button(icon::PREV_KEYFRAME.to_string()).clicked() {
        return Some(PlayControl::Prev);
    }

    if matches!(state, PlayState::PlayReverse) {
        if ui.button(icon::PAUSE.to_string()).clicked() {
            return Some(PlayControl::Pause);
        }
    } else if ui.button(icon::PLAY_REVERSE.to_string()).clicked() {
        return Some(PlayControl::PlayReverse);
    }

    if matches!(state, PlayState::Play) {
        if ui.button(icon::PAUSE.to_string()).clicked() {
            return Some(PlayControl::Pause);
        }
    } else if ui.button(icon::PLAY.to_string()).clicked() {
        return Some(PlayControl::Play);
    }

    if ui.button(icon::NEXT_KEYFRAME.to_string()).clicked() {
        return Some(PlayControl::Next);
    }
    if ui.button(icon::TRIA_RIGHT_BAR.to_string()).clicked() {
        return Some(PlayControl::Last);
    }
    None
}

impl BoneTimeline {
    fn show_translate(&self) -> bool {
        !self.translate.is_empty()
    }

    fn show_rotate(&self) -> bool {
        !self.rotate.is_empty()
    }

    fn show_scale(&self) -> bool {
        !self.scale.is_empty()
    }

    fn show_shear(&self) -> bool {
        !self.shear.is_empty()
    }

    fn extra_lines(&self) -> usize {
        self.show_rotate() as usize
            + self.show_translate() as usize
            + self.show_scale() as usize
            + self.show_shear() as usize
    }

    fn draw(&mut self, ui: &mut egui::Ui, style: &Style, current_time: u32) {
        let extra = self.extra_lines();
        if extra == 0 {
            return;
        }

        let id = Id::new(&self.label).with("_bone");

        let lines = 1 + if self.open { extra } else { 0 };
        let max_width = ui.available_size_before_wrap().x;
        let desired_size = vec2(max_width, LINE_HEIGHT * lines as f32);

        let (full_rect, _full_response) = ui.allocate_at_least(desired_size, egui::Sense::hover());

        for line in 0..lines {
            let y = full_rect.min.y + LINE_HEIGHT * line as f32;
            let line_rect = Rect {
                min: pos2(full_rect.min.x, y),
                max: pos2(full_rect.max.x, y + LINE_HEIGHT),
            };
            let line_response =
                ui.interact(line_rect, id.with("line_bg").with(line), Sense::hover());
            if line_response.hovered() {
                ui.painter()
                    .rect_filled(line_rect, 0.0, BONE_BG_COLOR_HOVER);
            }
        }

        let left_split = full_rect.min.x + LEFT_PADDING;

        let left_rect = full_rect.intersect(Rect::everything_left_of(left_split));
        let middle_rect = full_rect.intersect(Rect::everything_right_of(left_split));

        draw_grid(ui, middle_rect, current_time);

        {
            let bone_title_rect =
                left_rect.intersect(Rect::everything_above(left_rect.min.y + LINE_HEIGHT));

            let response = ui.interact(bone_title_rect, id.with("_bone_title"), Sense::click());
            let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
            if response.clicked() {
                self.open = !self.open;
            }

            let open_icon = if self.open {
                icon::DISCLOSURE_TRI_DOWN
            } else {
                icon::DISCLOSURE_TRI_RIGHT
            };

            let offset = bone_title_rect.left_center();

            let pos = offset + vec2(8.0, 0.0);
            draw_icon_at(ui, pos, style.input_text, 14.0, open_icon);
            let pos = offset + vec2(22.0, 0.0);
            draw_icon_at(ui, pos, style.input_text, 14.0, icon::BONE_DATA);

            let font_id = egui::FontId::proportional(14.0);
            let text = self.label.clone();
            let galley = ui.painter().layout_no_wrap(text, font_id, style.input_text);
            let pos = offset + vec2(32.0, -galley.size().y / 2.0);
            ui.painter().galley(pos, galley);
        }

        if self.open {
            let mut start = left_rect.min + vec2(32.0, LINE_HEIGHT);

            fn draw_label(ui: &mut Ui, start: Pos2, color: Color32, text: &str) {
                let font_id = egui::FontId::proportional(14.0);
                let galley = ui.painter().layout_no_wrap(text.into(), font_id, color);
                let pos = start + vec2(0.0, (LINE_HEIGHT - galley.size().y) / 2.0);
                ui.painter().galley(pos, galley);
            }

            if self.show_rotate() {
                draw_label(ui, start, ROTATE_COLOR, "rotate");
                start.y += LINE_HEIGHT;
            }
            if self.show_translate() {
                draw_label(ui, start, TRANSLATE_COLOR, "translate");
                start.y += LINE_HEIGHT;
            }
            if self.show_scale() {
                draw_label(ui, start, SCALE_COLOR, "scale");
                start.y += LINE_HEIGHT;
            }
            if self.show_shear() {
                draw_label(ui, start, SHEAR_COLOR, "shear");
                start.y += LINE_HEIGHT;
            }
        }

        self.update_keys();
        for &key in &self.keys {
            let x = middle_rect.min.x + ROW_WIDTH * key as f32;

            let rect = egui::Rect {
                min: egui::pos2(x - ROW_WIDTH / 2.0, middle_rect.min.y),
                max: egui::pos2(x + ROW_WIDTH / 2.0, middle_rect.min.y + LINE_HEIGHT),
            };

            let rect = rect.shrink(1.0);

            let id = id.with("_marker").with(key);
            let response = ui
                .interact(rect, id, Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);

            if response.clicked() {
                //key.hlt = !key.hlt;
            }
            let is_hovered = response.hovered();

            let color = if is_hovered {
                KEYFRAME_COLOR_HOVER
            } else {
                KEYFRAME_COLOR
            };

            let pos = rect.center();
            draw_icon_at(ui, pos, color, KEYFRAME_SIZE, icon::KEYFRAME_HLT);
        }

        if self.open {
            let mut start = middle_rect.min + vec2(0.0, LINE_HEIGHT);

            let px = ui.ctx().pixels_per_point().recip();

            fn widget(ui: &mut Ui, start: Pos2, curr: u32, id: Id, color: Color32) {
                let curr = ROW_WIDTH * curr as f32;
                let rect = egui::Rect {
                    min: start + vec2(curr - ROW_WIDTH / 2.0, 0.0),
                    max: start + vec2(curr + ROW_WIDTH / 2.0, LINE_HEIGHT),
                };

                let rect = rect.shrink(1.0);

                let response = ui
                    .interact(rect, id, Sense::click_and_drag())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                draw_bar(ui, response.rect, color);
            }

            if self.show_rotate() {
                for curr_index in 0..self.rotate.len() {
                    let (current, curve, next) = self.rotate.get(curr_index);
                    let id = id.with("rotate").with(current);
                    draw_curve(px, ui, start, current, next, curve);
                    widget(ui, start, current, id, ROTATE_COLOR);
                }
                start.y += LINE_HEIGHT;
            }

            if self.show_translate() {
                for curr_index in 0..self.translate.len() {
                    let (current, curve, next) = self.translate.get(curr_index);
                    let id = id.with("translate").with(current);
                    draw_curve(px, ui, start, current, next, curve);
                    widget(ui, start, current, id, TRANSLATE_COLOR);
                }
                start.y += LINE_HEIGHT;
            }

            if self.show_scale() {
                for curr_index in 0..self.scale.len() {
                    let (current, curve, next) = self.scale.get(curr_index);
                    let id = id.with("scale").with(current);
                    draw_curve(px, ui, start, current, next, curve);
                    widget(ui, start, current, id, SCALE_COLOR);
                }
                start.y += LINE_HEIGHT;
            }

            if self.show_shear() {
                for curr_index in 0..self.shear.len() {
                    let (current, curve, next) = self.shear.get(curr_index);
                    let id = id.with("shear").with(current);
                    draw_curve(px, ui, start, current, next, curve);
                    widget(ui, start, current, id, SHEAR_COLOR);
                }
                start.y += LINE_HEIGHT;
            }
        }
    }
}

fn draw_grid(ui: &mut Ui, rect: Rect, current_time: u32) {
    let ppi = ui.ctx().pixels_per_point();
    let px = ppi.recip();

    let count = (rect.width() / ROW_WIDTH) as u32;

    let painter = ui.painter();
    for i in 0..=count {
        let x = rect.min.x + ROW_WIDTH * i as f32;
        let x = crate::util::map_to_pixel(x, ppi, f32::floor);
        let a = egui::pos2(x, rect.min.y);
        let b = egui::pos2(x, rect.max.y);

        let color = if i == current_time {
            KEY_DIVIDER_CURRENT_COLOR
        } else {
            KEY_DIVIDER_COLOR
        };
        painter.line_segment([a, b], (px, color));
    }
}

fn draw_icon_at(ui: &mut Ui, pos: Pos2, color: Color32, size: f32, icon: char) {
    let font_id = egui::FontId::proportional(size);
    let text = icon.to_string();
    let galley = ui.painter().layout_no_wrap(text, font_id, color);
    let pos = pos - galley.size() / 2.0;
    ui.painter().galley(pos, galley);
}

fn draw_bar(ui: &mut Ui, rect: Rect, color: Color32) {
    let px = ui.ctx().pixels_per_point().recip();

    let sx = rect.width() / 2.0 - 0.5 - px;
    let bar_rect = rect.shrink2(vec2(sx, 0.0));
    ui.painter().rect_filled(bar_rect, 0.0, color);
}

fn draw_curve<T>(
    px: f32,
    ui: &mut Ui,
    start: Pos2,
    current: u32,
    next: Option<u32>,
    curve: Interpolation<T>,
) {
    if let Some(next) = next {
        let current = ROW_WIDTH * current as f32;
        let next = ROW_WIDTH * next as f32;

        let a = start + vec2(current, LINE_HEIGHT - 1.0);
        let b = start + vec2(next, 1.0);
        let color = CURVE_COLOR.linear_multiply(CURVE_COLOR_FACTOR);
        match curve {
            Interpolation::Linear => ui.painter().line_segment([a, b], (px, color)),
            Interpolation::Spline(_, _) => {
                let shape = egui::epaint::CubicBezierShape {
                    points: [a, a - vec2(0.0, LINE_HEIGHT), b + vec2(0.0, LINE_HEIGHT), b],
                    closed: false,
                    fill: Color32::TRANSPARENT,
                    stroke: (px, color).into(),
                };
                ui.painter().add(shape);
            }
        }
    }
}
