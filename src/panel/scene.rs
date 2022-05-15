use crate::app::TabInner;
use crate::asset::ReflectScene;
use crate::component::{ProxyPointLight, ProxyTransform};
use crate::context::{EditorContext, ReflectEntityGetters};
use crate::style::Style;
use bevy::prelude::*;
use bevy::render::camera::CameraProjection;
use bevy::utils::HashMap;

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
    texture_id: Option<egui::TextureId>,
    mapping: HashMap<u32, usize>,
    transform: HashMap<usize, GlobalTransform>,

    camera_proj: PerspectiveProjection,
    camera_view: Transform,
}

impl Default for SceneTab {
    fn default() -> Self {
        Self {
            texture_id: None,
            mapping: HashMap::default(),
            transform: HashMap::default(),

            camera_proj: PerspectiveProjection::default(),
            camera_view: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)),
        }
    }
}

impl TabInner for SceneTab {
    fn ui(&mut self, ui: &mut egui::Ui, _style: &Style, mut ctx: EditorContext) {
        {
            self.mapping.clear();
            self.mapping.extend(
                ctx.scene
                    .entities
                    .iter()
                    .enumerate()
                    .map(|(index, entity)| (entity.entity, index)),
            );

            fn recursive_propagate(
                mapping: &HashMap<u32, usize>,
                transform_mapping: &mut HashMap<usize, GlobalTransform>,
                scene: &ReflectScene,
                parent_index: usize,
            ) -> Option<()> {
                let parent = scene.entities.get(parent_index).unwrap();
                let parent_transform = transform_mapping[&parent_index];

                for child in parent
                    .children()?
                    .iter()
                    .filter_map(|entity| entity.downcast_ref::<Entity>())
                {
                    if let Some(&index) = mapping.get(&child.id()) {
                        if let Some(child) = scene.entities.get(index) {
                            if let Some(transform) = child.component_read::<ProxyTransform>() {
                                transform_mapping.insert(
                                    index,
                                    parent_transform.mul_transform(transform.to_local()),
                                );
                                recursive_propagate(mapping, transform_mapping, scene, index);
                            }
                        }
                    }
                }

                None
            }

            self.transform.clear();

            for parent_index in 0..ctx.scene.entities.len() {
                let parent = ctx.scene.entities.get(parent_index).unwrap();
                if parent.without::<Parent>() {
                    if let Some(parent_transform) = parent.component_read::<ProxyTransform>() {
                        self.transform
                            .insert(parent_index, parent_transform.to_global());

                        recursive_propagate(
                            &self.mapping,
                            &mut self.transform,
                            ctx.scene,
                            parent_index,
                        );
                    }
                }
            }
        }

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
                let pan_speed = 15.0;
                let rot_speed = 4.0;

                self.camera_proj.update(size_px.x, size_px.y);

                if let Some(drag) = state.drag.take() {
                    let delta = drag.delta / size_px;
                    match drag.button {
                        egui::PointerButton::Middle => {
                            let ratio = self.camera_proj.aspect_ratio;
                            let fov = self.camera_proj.fov;
                            let pan = delta * egui::Vec2::new(fov * ratio, fov);
                            let right = self.camera_view.rotation * Vec3::X * -pan.x;
                            let up = self.camera_view.rotation * Vec3::Y * pan.y;
                            self.camera_view.translation += (right + up) * pan_speed;
                        }
                        egui::PointerButton::Secondary => {
                            let delta = -delta * rot_speed;
                            let x = Quat::from_axis_angle(Vec3::Y, delta.x);
                            let y = Quat::from_axis_angle(Vec3::X, delta.y);
                            self.camera_view.rotation =
                                (self.camera_view.rotation * (x * y)).normalize();
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
                    if let Some(transform) = self.transform.get(&index) {
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

pub fn update_scene_render_target(
    mut tree: ResMut<crate::app::SplitTree>,
    mut egui_context: ResMut<crate::shell::EguiContext>,
    scene_render_target: Res<SceneRenderTarget>,
    mut images: ResMut<Assets<Image>>,
    mut camera: Query<
        (&mut Transform, &mut PerspectiveProjection),
        With<bevy::render::camera::Camera3d>,
    >,
) {
    let [ctx] = egui_context.ctx_mut([bevy::window::WindowId::primary()]);

    if let Some(handle) = scene_render_target.0.as_ref() {
        if let Some(image) = images.get_mut(handle) {
            if let Some((viewport, tab)) = tree.find_active::<SceneTab>() {
                let width = viewport.width() * ctx.pixels_per_point();
                let height = viewport.height() * ctx.pixels_per_point();
                image.resize(wgpu::Extent3d {
                    width: width as u32,
                    height: height as u32,
                    ..default()
                });

                tab.texture_id = Some(egui_context.add_image(handle.clone_weak()));

                for (mut transform, mut projection) in camera.iter_mut() {
                    *transform = tab.camera_view;
                    *projection = tab.camera_proj.clone();
                }

                /*
                let mov_speed = 8.0;
                let pan_speed = 15.0;
                let rot_speed = 4.0;

                if let Some(input) = unsafe { state.query::<Option<&mut InputState>>() } {
                    for (mut transform, projection) in camera.iter_mut() {
                        if let Some(drag) = input.drag.take() {
                            let delta = drag.delta / egui::vec2(width, height);
                            match drag.button {
                                egui::PointerButton::Middle => {
                                    let ratio = projection.aspect_ratio;
                                    let fov = projection.fov;
                                    let pan = delta * egui::Vec2::new(fov * ratio, fov);
                                    let right = transform.rotation * Vec3::X * -pan.x;
                                    let up = transform.rotation * Vec3::Y * pan.y;
                                    transform.translation += (right + up) * pan_speed;
                                }
                                egui::PointerButton::Secondary => {
                                    let delta = -delta * rot_speed;
                                    let x = Quat::from_axis_angle(Vec3::Y, delta.x);
                                    let y = Quat::from_axis_angle(Vec3::X, delta.y);
                                    transform.rotation = (transform.rotation * (x * y)).normalize();
                                }
                                _ => (),
                            }
                        }

                        let mut movement = Vec3::ZERO;

                        movement -= Vec3::Z * if input.forward { 1.0 } else { 0.0 };
                        movement += Vec3::Z * if input.backward { 1.0 } else { 0.0 };
                        movement -= Vec3::X * if input.left { 1.0 } else { 0.0 };
                        movement += Vec3::X * if input.right { 1.0 } else { 0.0 };

                        movement = movement.normalize_or_zero();
                        if input.hover_pos.is_some() {
                            movement -= Vec3::Z * input.scroll;
                        }

                        let movement = transform.rotation * movement;
                        transform.translation += movement * time.delta_seconds() * mov_speed;
                    }
                }
                */
            }
        }
    }
}
