use crate::app::{BackendError, RenderPass, ScreenDescriptor};
use ahash::AHashSet;
use egui::*;
use std::hash::Hash;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    window::Window,
};

pub struct Panel {
    pub context: Context,
    pub state: egui_winit::State,
    pub renderpass: RenderPass,
    pub zoom_level: f32,
    pub zoom_delta: f32,
    pub zoom_fix: egui::Pos2,
    pub mouse: MouseInput,
}

impl Panel {
    pub fn new(
        device: &wgpu::Device,
        window: &Window,
        output_format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> Self {
        let limits = device.limits();
        let max_texture_side = limits.max_texture_dimension_2d as usize;

        Self {
            state: egui_winit::State::new(max_texture_side, window),
            renderpass: RenderPass::new(device, output_format, sample_count),
            context: {
                let context = Context::default();
                context.set_fonts(super::fonts_with_blender());
                context
            },
            mouse: MouseInput::default(),
            zoom_delta: 0.0,
            zoom_level: 1.0,
            zoom_fix: egui::Pos2::ZERO,
        }
    }

    pub fn begin(&mut self, window: &Window, viewport: Rect) {
        let mut new_input = self.state.take_egui_input(window);

        let scale = window.scale_factor() as f32;

        let pixels_per_point = {
            let scroll_y = new_input.events.iter().filter_map(|event| match event {
                Event::Scroll(Vec2 { y, .. }) => Some(y),
                _ => None,
            });

            let next = (self.zoom_level + scroll_y.sum::<f32>() * 0.001).clamp(0.5, 4.0);
            self.zoom_delta = self.zoom_level - next;
            self.zoom_level = next;

            self.zoom_level * scale
        };

        if !self.context.wants_pointer_input() {
            new_input.pixels_per_point = Some(pixels_per_point);
        }

        let viewport = rect_scale(viewport, scale / pixels_per_point);
        new_input.screen_rect = Some(viewport);
        self.zoom_fix = viewport.min;

        self.context.begin_frame(new_input);
    }

    pub fn end(
        &mut self,
        super::RenderContext {
            device,
            queue,
            window,
            encoder,
            attachment,
            viewport,
        }: super::RenderContext,
    ) -> Result<bool, BackendError> {
        let FullOutput {
            shapes,
            needs_repaint,
            textures_delta,
            platform_output,
        } = self.context.end_frame();

        self.state
            .handle_platform_output(window, &self.context, platform_output);

        let size = window.inner_size();

        let screen_descriptor = ScreenDescriptor {
            width: size.width,
            height: size.height,
            scale: self.context.pixels_per_point(),
        };

        let paint_jobs = self.context.tessellate(shapes);

        self.renderpass
            .add_textures(device, queue, &textures_delta)?;
        self.renderpass.remove_textures(textures_delta)?;
        self.renderpass
            .update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui main render pass"),
            color_attachments: &[attachment],
            depth_stencil_attachment: None,
        });

        rpass.push_debug_group("egui_pass");

        self.renderpass
            .execute(&mut rpass, &paint_jobs, &screen_descriptor, viewport)?;

        rpass.pop_debug_group();

        self.mouse.update();

        Ok(needs_repaint)
    }

    pub fn on_event(&mut self, event: &WindowEvent, viewport: Rect, parent_scale: f32) {
        self.mouse.on_event(event, viewport, parent_scale);

        match event {
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => return,
            WindowEvent::CursorMoved { .. } if !self.mouse.in_viewport => return,
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } if !self.mouse.in_viewport => return,
            _ => (),
        }
        self.state.on_event(&self.context, event);
    }
}

/// Transforms a window-relative position `pos` into viewport relative
/// coordinates for a viewport at `viewport_rect`, with a `zoom_level` in a
/// window using a hiDPI scaling of `parent_scale`.
pub fn viewport_relative_position_winit(
    mut position: PhysicalPosition<f64>,
    parent_scale: f32,
    viewport: Rect,
    zoom_level: f32,
) -> PhysicalPosition<f64> {
    position.x -= (viewport.min.x * parent_scale) as f64;
    position.y -= (viewport.min.y * parent_scale) as f64;
    position.x *= zoom_level as f64;
    position.y *= zoom_level as f64;
    position
}

pub fn viewport_relative_position(
    mut position: Pos2,
    parent_scale: f32,
    viewport: Rect,
    zoom_level: f32,
) -> Pos2 {
    position.x -= viewport.min.x * parent_scale;
    position.y -= viewport.min.y * parent_scale;
    position.x *= zoom_level;
    position.y *= zoom_level;
    position
}

#[inline]
pub fn rect_scale(rect: Rect, scale: f32) -> Rect {
    Rect {
        min: (rect.min.to_vec2() * scale).to_pos2(),
        max: (rect.max.to_vec2() * scale).to_pos2(),
    }
}

#[derive(Default)]
pub struct InputSystem {
    pub mouse: MouseInput,
}

impl InputSystem {
    /// Called every frame, updates the input data structures
    pub fn update(&mut self) {
        self.mouse.update();
    }

    /// Called when a new `winit` window event is received. The `viewport_rect`
    /// and `parent_scaling` are used to translate mouse events to
    /// viewport-relative coordinates
    pub fn on_event(&mut self, event: &WindowEvent, viewport: Rect, parent_scale: f32) {
        self.mouse.on_event(event, viewport, parent_scale);
    }
}

#[derive(Default)]
pub struct MouseInput {
    buttons: Input<MouseButton>,
    last_pos: Option<Pos2>,
    last_pos_raw: Option<Pos2>,
    delta: Vec2,
    wheel_delta: f32,
    in_viewport: bool,
}

impl MouseInput {
    fn update(&mut self) {
        self.delta = Vec2::ZERO;
        self.wheel_delta = 0.0;
        self.buttons.update();
    }

    pub fn in_viewport(&self) -> bool {
        self.in_viewport
    }

    /// Get a reference to the mouse input's buttons.
    pub fn buttons(&self) -> &Input<MouseButton> {
        &self.buttons
    }

    /// Get a reference to the mouse input's last pos.
    pub fn position(&self) -> Option<Pos2> {
        self.last_pos
    }

    /// Get a reference to the mouse input's delta.
    pub fn cursor_delta(&self) -> Vec2 {
        self.delta
    }

    /// Get a reference to the mouse input's wheel delta.
    pub fn wheel_delta(&self) -> f32 {
        self.wheel_delta
    }

    /// Called when a new `winit` window event is received. The `viewport_rect`
    /// and `parent_scaling` are used to translate mouse events to
    /// viewport-relative coordinates
    pub fn on_event(&mut self, event: &WindowEvent, viewport: Rect, parent_scale: f32) {
        self.in_viewport = self.last_pos_raw.map_or(false, |position| {
            rect_scale(viewport, parent_scale).contains(position)
        });

        match event {
            // Cursor moves are always registered.
            // The raw (untransformed) mouse position is also stored
            // so we know if the mosue is over the viewport on the next events.
            WindowEvent::CursorMoved { position, .. } => {
                let position = Pos2::new(position.x as f32, position.y as f32);
                self.last_pos_raw = Some(position);

                // zoom doesn't affect cursor on this viewport
                let position = viewport_relative_position(position, parent_scale, viewport, 1.0);

                let last_pos = self.last_pos.unwrap_or(position);

                self.delta = position - last_pos;
                self.last_pos = Some(position);
            }

            // Wheel events will only get registered when the cursor is inside the viewport
            WindowEvent::MouseWheel { delta, .. } if self.in_viewport => match delta {
                MouseScrollDelta::LineDelta(_, y) => self.wheel_delta = *y as f32,
                MouseScrollDelta::PixelDelta(pos) => self.wheel_delta = pos.y as f32,
            },

            // Button events are a bit different: Presses can register inside
            // the viewport but releases will register anywhere.
            WindowEvent::MouseInput {
                button,
                state: ElementState::Pressed,
                ..
            } if self.in_viewport => self.buttons.press(*button),
            WindowEvent::MouseInput {
                button,
                state: ElementState::Released,
                ..
            } => self.buttons.release(*button),
            _ => {}
        }
    }
}

pub struct Input<Button> {
    pressed: AHashSet<Button>,
    just_pressed: AHashSet<Button>,
    just_released: AHashSet<Button>,
}

impl<Button: Clone + Copy + Eq + Hash> Default for Input<Button> {
    fn default() -> Self {
        Self {
            pressed: AHashSet::default(),
            just_pressed: AHashSet::default(),
            just_released: AHashSet::default(),
        }
    }
}

impl<Button: Clone + Copy + Eq + Hash> Input<Button> {
    fn press(&mut self, button: Button) {
        self.pressed.insert(button);
        self.just_pressed.insert(button);
    }

    fn release(&mut self, button: Button) {
        self.pressed.remove(&button);
        self.just_released.insert(button);
    }

    pub fn pressed(&self, button: Button) -> bool {
        self.pressed.contains(&button)
    }

    pub fn just_pressed(&self, button: Button) -> bool {
        self.just_pressed.contains(&button)
    }

    pub fn just_released(&self, button: Button) -> bool {
        self.just_released.contains(&button)
    }

    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}
