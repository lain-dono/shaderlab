use super::SceneMapping;
use crate::app::TabInner;
use crate::component::ProxyPointLight;
use crate::context::{EditorContext, ReflectEntityGetters};
use crate::style::Style;
use bevy::prelude::*;
use bevy::render::camera::CameraProjection;

struct Drag {
    delta: egui::Vec2,
    button: egui::PointerButton,
}

#[derive(Default)]
struct InputState {
    drag: Option<Drag>,
    hover_pos: Option<egui::Pos2>,
    modifiers: egui::Modifiers,

    scroll: f32,

    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

pub struct SceneTab {
    pub texture_id: Option<egui::TextureId>,
    pub camera_proj: PerspectiveProjection,
    pub camera_view: Transform,

    pub yew: f32,
    pub pitch: f32,
}

impl Default for SceneTab {
    fn default() -> Self {
        Self {
            texture_id: None,
            camera_proj: PerspectiveProjection::default(),
            camera_view: Transform::from_translation(Vec3::new(0.0, 5.0, 15.0)),

            yew: 0.0,
            pitch: 0.0,
        }
    }
}

impl TabInner for SceneTab {
    fn ui(&mut self, ui: &mut egui::Ui, _style: &Style, ctx: EditorContext) {
        let mapping = ctx.state.get::<SceneMapping>().unwrap();

        let scale = ui.ctx().pixels_per_point();
        let size_ui = ui.available_size_before_wrap();
        let size_px = size_ui * scale;

        if let Some(texture_id) = self.texture_id {
            let response = ui.add(egui::widgets::Image::new(texture_id, size_ui));
            let response = response.interact(egui::Sense::click_and_drag());

            let screen_rect = response.rect;
            {
                let mut state = InputState::default();
                if response.dragged_by(egui::PointerButton::Primary) {
                    state.drag = Some(Drag {
                        delta: response.drag_delta(),
                        button: egui::PointerButton::Primary,
                    });
                }
                if response.dragged_by(egui::PointerButton::Secondary) {
                    state.drag = Some(Drag {
                        delta: response.drag_delta(),
                        button: egui::PointerButton::Secondary,
                    });
                }
                if response.dragged_by(egui::PointerButton::Middle) {
                    state.drag = Some(Drag {
                        delta: response.drag_delta(),
                        button: egui::PointerButton::Middle,
                    });
                }

                state.hover_pos = response.hover_pos();

                let input = ui.input();
                state.modifiers = input.modifiers;
                state.scroll = input.scroll_delta.y;
                state.forward = input.key_down(egui::Key::W);
                state.backward = input.key_down(egui::Key::S);
                state.left = input.key_down(egui::Key::A);
                state.right = input.key_down(egui::Key::D);

                let mov_speed = 8.0;
                let pan_speed = 25.0;
                let rot_speed = 2.0;

                self.camera_proj.update(size_px.x, size_px.y);

                if let Some(drag) = state.drag.take() {
                    let delta = drag.delta / size_px;
                    let ratio = self.camera_proj.aspect_ratio;
                    let fov = self.camera_proj.fov;
                    match drag.button {
                        egui::PointerButton::Middle => {
                            let pan = delta * egui::Vec2::new(fov * ratio, fov);
                            let right = self.camera_view.rotation * Vec3::X * -pan.x;
                            let up = self.camera_view.rotation * Vec3::Y * pan.y;
                            self.camera_view.translation += (right + up) * pan_speed;
                        }
                        egui::PointerButton::Secondary => {
                            self.yew += delta.x * fov * ratio * rot_speed;
                            self.pitch += delta.y * fov * rot_speed;

                            self.camera_view.rotation =
                                Quat::from_euler(EulerRot::YXZ, self.yew, self.pitch, 0.0);
                        }
                        _ => (),
                    }
                }

                let mut movement = Vec3::ZERO;

                movement -= Vec3::Z * if state.forward { 1.0 } else { 0.0 };
                movement += Vec3::Z * if state.backward { 1.0 } else { 0.0 };
                movement -= Vec3::X * if state.left { 1.0 } else { 0.0 };
                movement += Vec3::X * if state.right { 1.0 } else { 0.0 };

                movement = movement.normalize_or_zero();
                if state.hover_pos.is_some() {
                    movement -= Vec3::Z * state.scroll;
                }

                let movement = self.camera_view.rotation * movement;
                self.camera_view.translation += movement * input.predicted_dt * mov_speed;
            }

            if true {
                let proj = self.camera_proj.get_projection_matrix();
                let view = self.camera_view.compute_matrix().inverse();
                let world_to_ndc = proj * view;

                let world_to_screen = |world_position: Vec3| {
                    let ndc = world_to_ndc.project_point3(world_position);

                    // NDC z-values outside of 0 < z < 1 are outside the camera frustum and are thus not in screen space
                    if ndc.is_nan() || ndc.z < 0.0 || ndc.z > 1.0 {
                        None
                    } else {
                        // Once in NDC space, we can discard the z element and rescale x/y to fit the screen
                        Some((
                            screen_rect.min + egui::vec2(ndc.x + 1.0, 1.0 - ndc.y) / 2.0 * size_ui,
                            1.0 - ndc.z,
                        ))
                    }
                };

                ui.set_clip_rect(screen_rect);
                let painter = ui.painter();
                for (index, entity) in ctx.scene.entities.iter_mut().enumerate() {
                    if let Some(transform) = mapping.transform.get(&index) {
                        if let Some((center, _z)) = world_to_screen(transform.translation) {
                            if entity.has::<ProxyPointLight>() {
                                painter.text(
                                    center,
                                    egui::Align2::CENTER_CENTER,
                                    crate::icon::LIGHT_POINT,
                                    egui::FontId::proportional(20.0),
                                    egui::Color32::WHITE,
                                );
                                continue;
                            }

                            let fill = egui::Color32::WHITE;
                            let stroke = (1.0, egui::Color32::BLACK);
                            painter.circle(center, 2.0, fill, stroke);
                        }
                    }
                }
            }
        }
    }
}

pub struct SceneRenderTarget(pub Option<Handle<Image>>);

impl SceneRenderTarget {
    pub fn insert(commands: &mut Commands, images: &mut Assets<Image>) -> Handle<Image> {
        use bevy::render::render_resource::*;

        let size = Extent3d {
            width: 1,
            height: 1,
            ..default()
        };

        // This is the texture that will be rendered to.
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..default()
        };

        // fill image.data with zeroes
        image.resize(size);

        let handle = images.add(image);

        commands.insert_resource(Self(Some(handle.clone())));

        handle
    }
}
