use super::armature::paint_bones;
use super::{
    Animation, Armature, Bone, Controller, Grid, Keyframe, Matrix, Timeline, TimelineValue,
    Transform,
};
use crate::ui::{icon, EditorTab, PanelRenderTarget, Style};
use bevy::ecs::entity::Entity;
use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes, SResMut};
use bevy::ecs::system::SystemParamItem;
use egui::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SetupMode {
    Pose,
    Edit,
    Weight,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Setup,
    Animate,
}

#[derive(bevy::prelude::Component)]
pub struct Animation2d {
    pub setup_mode: SetupMode,
    pub mode: Mode,
    pub grid: Grid,
}

impl Default for Animation2d {
    fn default() -> Self {
        Self {
            setup_mode: SetupMode::Pose,
            mode: Mode::Animate,
            grid: Grid::default(),
        }
    }
}

impl EditorTab for Animation2d {
    type Param = (
        SQuery<Read<PanelRenderTarget>>,
        SRes<Style>,
        SResMut<Armature>,
        SResMut<Animation>,
        SResMut<Controller>,
    );

    fn ui<'w>(
        &mut self,
        ui: &mut egui::Ui,
        entity: Entity,
        (target, style, armature, animation, controller): &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        let frame = ui.available_rect_before_wrap();
        ui.painter().rect_filled(frame, 0.0, style.panel);

        let inner_frame = frame.shrink(16.0);

        let pointer = ui
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

            self.grid.update(ui, frame);
            let viewport = self.grid.viewport(pointer, frame);

            let hovered = {
                controller.world_to_screen(viewport.matrix());

                let mut shapes = Vec::new();
                self.grid.paint(ui, frame, pointer, &mut shapes);

                if let Some(texture_id) = target.get(entity).ok().and_then(|t| t.texture_id) {
                    let rect = frame;
                    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
                    let mut mesh = Mesh::with_texture(texture_id);
                    mesh.add_rect_with_uv(rect, uv, Color32::WHITE);
                    ui.painter().add(Shape::mesh(mesh));
                }

                let hovered = paint_bones(ui, viewport, armature, controller, &mut shapes);
                ui.painter().extend(shapes);

                hovered
            };

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
                        let text = format!("{} Pose", icon::POSE_HLT);
                        ui[0].selectable_value(&mut self.setup_mode, SetupMode::Pose, text);
                        let text = format!("{} Edit", icon::EDITMODE_HLT);
                        ui[1].selectable_value(&mut self.setup_mode, SetupMode::Edit, text);
                        let text = format!("{} Weight", icon::WPAINT_HLT);
                        ui[2].selectable_value(&mut self.setup_mode, SetupMode::Weight, text);
                    });
                });
            });

            let cursor_in_mode = response.hover_pos().is_some();

            let cursor_in_controls =
                if let Some(bone) = controller.selected.map(|index| &mut armature.bones[index]) {
                    frame
                        .show(&mut controls_ui, |ui| {
                            transform_widget(ui, bone, controller, animation, self.mode)
                        })
                        .response
                        .hover_pos()
                        .is_some()
                } else {
                    false
                };

            let cursor_in_ui = cursor_in_mode || cursor_in_controls;

            if pointer.is_some() && !cursor_in_ui {
                ui.ctx().output().cursor_icon = CursorIcon::Crosshair;
                if ui.input().pointer.any_click() {
                    controller.selected = hovered;
                }
            }
        });
    }
}

fn transform_widget(
    ui: &mut Ui,
    bone: &mut Bone,
    controller: &Controller,
    animation: &mut Animation,
    mode: Mode,
) -> InnerResponse<()> {
    let index = controller.selected.unwrap();
    let time = controller.current_time;

    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        ui.horizontal(|ui| {
            ui.add_space(2.0);

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 5.0);
                ui.add_space(2.0);
                ui.label("rotate");
                ui.label("translate");
                ui.label("scale");
                ui.label("shear");
            });

            ui.add_space(4.0);

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(1.0, 1.0);

                match mode {
                    Mode::Setup => bone.transform = edit_transform(ui, bone.transform),
                    Mode::Animate => {
                        let clip = animation.bones[index].resolve(time as f32);
                        let transform = bone.transform.mul_transform(clip);
                        let transform = edit_transform(ui, transform);
                        //bone.transform = transform.mul_transform(clip.inverse());
                    }
                };
            });

            ui.vertical(|ui| {
                fn toggle_key<T: TimelineValue>(
                    ui: &mut Ui,
                    time: u32,
                    timeline: &mut Timeline<T>,
                    default: T,
                    pose: T,
                ) {
                    let cond = timeline.at(time).is_some();
                    let icon = if cond { icon::KEYFRAME_HLT } else { icon::DOT };
                    let btn = Button::new(icon.to_string()).frame(false);
                    if ui.add(btn).clicked() {
                        if cond {
                            timeline.remove(time);
                        } else {
                            let value = timeline.resolve_or(time as f32, default);
                            timeline.add(Keyframe::linear(time, value));
                        }
                    }
                }

                let clip = &mut animation.bones[index];

                ui.spacing_mut().item_spacing = vec2(0.0, 1.0);
                ui.add_space(2.0);

                let pose = bone.transform.rotate;
                toggle_key(ui, time, &mut clip.rotate, 0.0, pose);

                let pose = bone.transform.translate.to_vec2();
                toggle_key(ui, time, &mut clip.translate, vec2(0.0, 0.0), pose);

                let pose = bone.transform.scale;
                toggle_key(ui, time, &mut clip.scale, vec2(1.0, 1.0), pose);

                let pose = bone.transform.shear;
                toggle_key(ui, time, &mut clip.shear, vec2(0.0, 0.0), pose);
            });
        });
    })
}

fn edit_transform(ui: &mut Ui, mut transform: Transform) -> Transform {
    ui.columns(1, |ui| {
        ui[0].drag_angle(&mut transform.rotate);
    });

    ui.columns(2, |ui| {
        ui[0].add(DragValue::new(&mut transform.translate.x).speed(1.0));
        ui[1].add(DragValue::new(&mut transform.translate.y).speed(1.0));
    });

    ui.columns(2, |ui| {
        ui[0].add(DragValue::new(&mut transform.scale.x).speed(0.1));
        ui[1].add(DragValue::new(&mut transform.scale.y).speed(0.1));
    });

    ui.columns(2, |ui| {
        ui[0].drag_angle(&mut transform.shear.x);
        ui[1].drag_angle(&mut transform.shear.y);
    });

    transform
}
