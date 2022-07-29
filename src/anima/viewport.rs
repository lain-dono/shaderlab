use super::grid::Grid;
use super::{Armature, Controller};
use crate::app::TabInner;
use crate::context::EditorContext;
use crate::style::Style;
use egui::*;

#[derive(PartialEq)]
enum Mode {
    Edit,
    Pose,
    Weight,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Edit
    }
}

pub struct Animation2d {
    rotation: f32,
    location: Vec2,
    scale: Vec2,
    shear: Vec2,

    rotation_key: bool,
    location_key: bool,
    scale_key: bool,
    shear_key: bool,

    mode: Mode,

    grid: Grid,
    armature: Armature,
    controller: Controller,
}

impl Default for Animation2d {
    fn default() -> Self {
        Self {
            rotation: 0.0,
            location: vec2(0.0, 0.0),
            scale: vec2(1.0, 1.0),
            shear: vec2(0.0, 0.0),

            rotation_key: false,
            location_key: false,
            scale_key: false,
            shear_key: false,

            mode: Mode::Edit,

            grid: Grid::new(),
            controller: Controller::default(),
            armature: super::example::armature(),
        }
    }
}

impl TabInner for Animation2d {
    fn ui(&mut self, ui: &mut Ui, style: &Style, _ctx: EditorContext) {
        let frame = ui.available_rect_before_wrap();
        ui.painter().rect_filled(frame, 0.0, style.panel);

        let inner_frame = frame.shrink(16.0);

        let cursor = ui
            .input()
            .pointer
            .hover_pos()
            .filter(|&p| frame.contains(p));

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            ui.spacing_mut().item_spacing.y = 2.0;
            style.set_theme_visuals(ui);

            ui.set_clip_rect(frame);

            let ppi = ui.ctx().pixels_per_point();
            let px = ppi.recip();

            {
                self.grid.update(ui, frame);

                {
                    let offset = vec2(-self.grid.offset.x, self.grid.offset.y);
                    let zoom = self.grid.zoom_factor;
                    let offset = frame.center() + offset * zoom;

                    for bone in &mut self.armature.bones {
                        bone.rotation = ui.input().time as f32;
                    }

                    self.controller.collect_world_transform(&self.armature);

                    self.armature
                        .paint_bones(ui.painter(), offset, zoom, &self.controller.world);
                }

                let mut shapes = Vec::new();
                self.grid.paint(ui, frame, cursor, &mut shapes);
                ui.painter().extend(shapes);
            }

            let frame = egui::Frame::none()
                .inner_margin(2.0)
                .rounding(2.0)
                .fill(style.panel)
                .stroke(Stroke {
                    width: px,
                    color: Color32::from_gray(0x19),
                });

            let size = vec2(220.0, 25.0);
            let mut mode_ui = ui.child_ui_with_id_source(
                Align2::LEFT_TOP.align_size_within_rect(size, inner_frame),
                Layout::left_to_right(),
                Id::new("animation2d::mode_select"),
            );

            let size = vec2(180.0, 80.0);
            let mut controls_ui = ui.child_ui_with_id_source(
                Align2::CENTER_BOTTOM.align_size_within_rect(size, inner_frame),
                Layout::top_down(Align::Center),
                Id::new("animation2d::bottom_control"),
            );

            let InnerResponse { response, .. } = frame.show(&mut mode_ui, |ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                    ui.columns(3, |ui| {
                        let text = format!("{} Pose", crate::icon::POSE_HLT);
                        ui[0].selectable_value(&mut self.mode, Mode::Pose, text);
                        let text = format!("{} Edit", crate::icon::EDITMODE_HLT);
                        ui[1].selectable_value(&mut self.mode, Mode::Edit, text);
                        let text = format!("{} Weight", crate::icon::WPAINT_HLT);
                        ui[2].selectable_value(&mut self.mode, Mode::Weight, text);
                    });
                });
            });

            let cursor_in_mode = response.hover_pos().is_some();

            let InnerResponse { response, .. } = frame.show(&mut controls_ui, |ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                    ui.horizontal(|ui| {
                        ui.add_space(2.0);

                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 5.0);
                            ui.add_space(2.0);
                            ui.label("rotation");
                            ui.label("location");
                            ui.label("scale");
                            ui.label("shear");
                        });

                        ui.add_space(4.0);

                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = vec2(1.0, 1.0);
                            ui.columns(1, |ui| {
                                ui[0].drag_angle(&mut self.rotation);
                            });

                            ui.columns(2, |ui| {
                                ui[0].add(DragValue::new(&mut self.location.x).speed(0.1));
                                ui[1].add(DragValue::new(&mut self.location.y).speed(0.1));
                            });

                            ui.columns(2, |ui| {
                                ui[0].add(DragValue::new(&mut self.scale.x).speed(0.1));
                                ui[1].add(DragValue::new(&mut self.scale.y).speed(0.1));
                            });

                            ui.columns(2, |ui| {
                                ui[0].add(DragValue::new(&mut self.shear.x).speed(0.1));
                                ui[1].add(DragValue::new(&mut self.shear.y).speed(0.1));
                            });
                        });

                        ui.vertical(|ui| {
                            fn toggle_key(ui: &mut Ui, cond: &mut bool) {
                                let icon = if *cond {
                                    crate::icon::KEYFRAME_HLT
                                } else {
                                    crate::icon::DOT
                                };

                                let btn = Button::new(icon.to_string()).small().frame(false);
                                if ui.add(btn).clicked() {
                                    *cond = !*cond;
                                }
                            }

                            ui.spacing_mut().item_spacing = vec2(0.0, 5.0);
                            ui.add_space(2.0);
                            toggle_key(ui, &mut self.rotation_key);
                            toggle_key(ui, &mut self.location_key);
                            toggle_key(ui, &mut self.scale_key);
                            toggle_key(ui, &mut self.shear_key);
                        });
                    });
                });
            });

            let cursor_in_controls = response.hover_pos().is_some();

            if cursor.is_some() && !cursor_in_mode && !cursor_in_controls {
                ui.ctx().output().cursor_icon = CursorIcon::Crosshair;
            }
        });
    }
}
