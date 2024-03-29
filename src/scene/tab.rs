use super::context::{EditorContext, ReflectEntityGetters};
use super::{component::ProxyPointLight, ReflectScene, SceneMapping};
use crate::ui::{icon, EditorPanel, EditorTab, PanelRenderTarget};
use crate::util::anymap::AnyMap;
use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes, SResMut, Write};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::render::camera::{CameraProjection, Projection};

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

#[derive(Component)]
pub struct SceneTab {
    pub yew: f32,
    pub pitch: f32,
    pub zsorting: Vec<(f32, egui::Pos2, usize)>,
}

impl EditorTab for SceneTab {
    type Param = (
        SRes<Handle<ReflectScene>>,
        SResMut<AnyMap>,
        SResMut<Assets<ReflectScene>>,
        SRes<TypeRegistry>,
        SResMut<AssetServer>,
        SQuery<(Write<Transform>, Write<Projection>, Read<PanelRenderTarget>)>,
    );

    fn ui<'w>(
        &mut self,
        ui: &mut egui::Ui,
        entity: Entity,
        (scene, state, scenes, types, assets, query): &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        let scene = scenes.get_mut(scene).unwrap();
        let mut ctx = EditorContext {
            scene,
            state,
            types,
            assets,
        };

        let (mut camera_view, mut camera_proj, target) = query.get_mut(entity).unwrap();

        let scale = ui.ctx().pixels_per_point();
        let size_ui = ui.available_size_before_wrap();
        let size_px = size_ui * scale;

        if let Some(texture_id) = target.texture_id {
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

                camera_proj.update(size_px.x, size_px.y);

                if let Projection::Perspective(proj) = camera_proj.as_mut() {
                    if let Some(drag) = state.drag.take() {
                        let delta = drag.delta / size_px;
                        let ratio = proj.aspect_ratio;
                        let fov = proj.fov;
                        match drag.button {
                            egui::PointerButton::Middle => {
                                let pan = delta * egui::Vec2::new(fov * ratio, fov);
                                let right = camera_view.rotation * Vec3::X * -pan.x;
                                let up = camera_view.rotation * Vec3::Y * pan.y;
                                camera_view.translation += (right + up) * pan_speed;
                            }
                            egui::PointerButton::Secondary => {
                                self.yew += delta.x * fov * ratio * rot_speed;
                                self.pitch += delta.y * fov * rot_speed;

                                camera_view.rotation =
                                    Quat::from_euler(EulerRot::YXZ, self.yew, self.pitch, 0.0);
                            }
                            _ => (),
                        }
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

                let movement = camera_view.rotation * movement;
                camera_view.translation += movement * input.predicted_dt * mov_speed;
            }

            let proj = camera_proj.get_projection_matrix();
            let view = camera_view.compute_matrix().inverse();
            let world_to_ndc = proj * view;
            let ndc_to_world = view.inverse() * proj.inverse();

            let world_to_screen_and_z = |world: Vec3| {
                let ndc = world_to_ndc.project_point3(world);

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

            let mapping = ctx.state.get::<SceneMapping>().unwrap();

            self.zsorting.clear();
            self.zsorting.extend(
                ctx.scene
                    .entities
                    .iter()
                    .enumerate()
                    .filter_map(|(index, _)| {
                        mapping
                            .transform
                            .get(&index)
                            .and_then(|transform| {
                                world_to_screen_and_z(transform.affine().translation.into())
                            })
                            .map(|(pos, z)| (z, pos, index))
                    }),
            );

            // reverse z-sorting
            self.zsorting
                .sort_by(|(az, _, _), (bz, _, _)| f32::total_cmp(bz, az));

            let world_to_screen = |world: Vec3| {
                let ndc = world_to_ndc.project_point3(world);

                // NDC z-values outside of 0 < z < 1 are outside the camera frustum and are thus not in screen space
                if ndc.is_nan() || ndc.z < 0.0 || ndc.z > 1.0 {
                    None
                } else {
                    // Once in NDC space, we can discard the z element and rescale x/y to fit the screen
                    Some(screen_rect.min + egui::vec2(ndc.x + 1.0, 1.0 - ndc.y) / 2.0 * size_ui)
                    // XXX: 1.0 - z
                }
            };

            let _screen_to_world = |screen: egui::Pos2, z: f32| {
                let local = ((screen - screen_rect.min) * 2.0) / size_ui;
                ndc_to_world.project_point3(Vec3::new(local.x - 1.0, 1.0 - local.y, 1.0 - z))
            };

            ui.set_clip_rect(screen_rect);

            let click_pos = if response.clicked() {
                response.hover_pos()
            } else {
                None
            };

            let mouse = ui.input().pointer.hover_pos();
            let selected = ctx.selected_index(None);
            let mapping = ctx.state.get::<SceneMapping>().unwrap();
            let mut next_select = None;

            for &(_z, center, index) in &self.zsorting {
                let entity = &mut ctx.scene.entities[index];

                let transform = &mapping.transform[&index];
                let size = egui::vec2(16.0, 16.0);
                let rect = egui::Rect::from_center_size(center, size);
                let inner = ui.allocate_rect(rect, egui::Sense::hover());

                let mut color = egui::Color32::WHITE;
                if inner.hovered() {
                    color = from_raw(super::gizmo::clrs::YELLOW);
                    if click_pos.is_some() {
                        next_select = Some(index);
                    }
                }

                fn from_raw([r, g, b, _]: [u8; 4]) -> egui::Color32 {
                    egui::Color32::from_rgb(r, g, b)
                }

                let painter = ui.painter();

                let is_selected = selected == Some(index);

                let get_pos = |dir, color| {
                    world_to_screen(Vec3::from(transform.affine().translation) + dir)
                        .map(|p| (p, (1.0, from_raw(color))))
                };

                if is_selected {
                    let positions = [
                        get_pos(Vec3::X, super::gizmo::clrs::X_AXIS),
                        get_pos(Vec3::Y, super::gizmo::clrs::Y_AXIS),
                        get_pos(Vec3::Z, super::gizmo::clrs::Z_AXIS),
                    ];

                    let closest = mouse.and_then(|mouse| {
                        [
                            positions[0].map(|(end, _)| dist(center, end, mouse)),
                            positions[1].map(|(end, _)| dist(center, end, mouse)),
                            positions[2].map(|(end, _)| dist(center, end, mouse)),
                        ]
                        .into_iter()
                        .enumerate()
                        .filter_map(|(index, d)| d.map(|d| (index, d)))
                        .min_by(|(_, a), (_, b)| f32::total_cmp(a, b))
                        .map(|(index, _)| index)
                    });

                    let cc = (1.0, from_raw(super::gizmo::clrs::YELLOW));

                    for (i, item) in positions.into_iter().enumerate() {
                        if let Some((end, stroke)) = item {
                            let stroke = if closest == Some(i) { cc } else { stroke };
                            painter.line_segment([center, end], stroke);
                        }
                    }

                    color = from_raw(super::gizmo::clrs::YELLOW)
                }

                if entity.has::<ProxyPointLight>() {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        icon::LIGHT_POINT,
                        egui::FontId::proportional(20.0),
                        color,
                    );
                } else {
                    let fill = color;
                    let stroke = (1.0, egui::Color32::BLACK);
                    painter.circle(center, 2.0, fill, stroke);
                }
            }

            if let Some(entity) = next_select {
                dbg!(entity);
                ctx.select(entity);
            }
        }
    }
}

fn dist(a: egui::Pos2, b: egui::Pos2, p: egui::Pos2) -> f32 {
    let ab = a - b;
    let ba = b - a;
    let pb = p - b;
    let pa = p - a;

    if ab.x * pb.x + ab.y * pb.y <= 0.0 {
        return (pb.x * pb.x + pb.y * pb.y).sqrt();
    }

    if ba.x * pa.x + ba.y * pa.y <= 0.0 {
        return (pa.x * pa.x + pa.y * pa.y).sqrt();
    }

    (ba.y * p.x - ba.x * p.y + b.x * a.y - b.y * a.x).abs() / (ab.y * ab.y + ab.x * ab.x).sqrt()
}

#[derive(Clone, Copy, Debug)]
struct Ray {
    origin: Vec3,
    dir: Vec3,
}

fn closest_distance_between_rays(a: Ray, b: Ray) -> f32 {
    let dp = b.origin - a.origin;

    let dot_a = a.dir.dot(a.dir);
    let dot_b = b.dir.dot(b.dir);
    let ab = a.dir.dot(b.dir);

    let det = ab * ab - dot_a * dot_b;

    if det.abs() > f32::MIN {
        let inv_det = det.recip();

        let dp_a = dp.dot(a.dir);
        let dp_b = dp.dot(b.dir);

        let a_t = inv_det * (dot_b * dp_a - ab * dp_b);
        let b_t = inv_det * (ab * dp_a - dot_a * dp_b);

        Vec3::length(dp + b.dir * b_t - a.dir * a_t)
    } else {
        let a = dp.cross(a.dir);
        (a.dot(a) / dot_a).sqrt()
    }
}

impl super::gizmo::Lines {
    fn translation_gizmo(&mut self, transform: &Transform, selected_axis: Option<u8>) {
        for i in 0..3 {
            let (dir, mut color) = match i {
                0 => (Vec3::X, super::gizmo::clrs::X_AXIS),
                1 => (Vec3::Y, super::gizmo::clrs::Y_AXIS),
                2 => (Vec3::Z, super::gizmo::clrs::Z_AXIS),
                _ => unreachable!(),
            };

            if Some(i) == selected_axis {
                color = super::gizmo::clrs::YELLOW;
            }

            self.line(transform.translation, transform.translation + dir, color);
        }
    }
}

impl SceneTab {
    pub fn spawn(commands: &mut Commands, images: &mut Assets<Image>) -> crate::ui::Tab {
        use bevy::core_pipeline::clear_color::ClearColorConfig;
        use bevy::prelude::*;

        let target = PanelRenderTarget::create_render_target(images);

        let gray = 0x2B as f32 / 255.0;
        let clear_color = Color::rgba(gray, gray, gray, 1.0);

        let entity = commands
            .spawn_bundle(Camera3dBundle {
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(clear_color),
                    ..default()
                },
                camera: Camera {
                    target,
                    ..default()
                },

                transform: Transform::from_translation(Vec3::new(0.0, 5.0, 15.0)),

                ..default()
            })
            .insert(Self {
                yew: 0.0,
                pitch: 0.0,

                zsorting: Vec::new(),
            })
            .insert(EditorPanel::default())
            .insert(PanelRenderTarget::default())
            .id();

        crate::ui::Tab::new(icon::VIEW3D, "Scene", entity)
    }
}
