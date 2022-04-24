use crate::app::{RenderPass, ScreenDescriptor};
use crate::global::Global;
use crate::hierarchy::Hierarchy;
use crate::inspector::Inspector;
use std::future::Future;
use std::iter;
use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

mod global;
mod hierarchy;
mod inspector;

mod app;
mod blender;
mod builder;
mod framebuffer;
mod graph;
mod nodes;
pub mod workspace;

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

use crate::framebuffer::Framebuffer;
use crate::workspace::Workspace;

pub struct Spawner<'a> {
    executor: async_executor::LocalExecutor<'a>,
}

impl<'a> Spawner<'a> {
    fn new() -> Self {
        Self {
            executor: async_executor::LocalExecutor::new(),
        }
    }

    #[allow(dead_code)]
    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'a) {
        self.executor.spawn(future).detach();
    }

    fn run_until_stalled(&self) {
        while self.executor.try_tick() {}
    }
}

fn enable_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let format = fmt::format()
        .without_time()
        .with_target(true)
        .with_source_location(true)
        .compact();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .event_format(format)
        .init();
}

fn main() {
    enable_tracing();

    let mut event_loop: EventLoop<()> = EventLoop::with_user_event();
    let window = WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("Shaderlab")
        .with_inner_size(PhysicalSize {
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let desc = wgpu::DeviceDescriptor {
        features: wgpu::Features::default(),
        limits: wgpu::Limits::default(),
        label: None,
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&desc, None)).unwrap();

    let surface_format = surface.get_preferred_format(&adapter).unwrap();
    let sample_count = 4;

    let mut fb = {
        let size = window.inner_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        Framebuffer::new(&device, surface, sample_count, config)
    };

    let mut app = {
        use crate::app::*;
        use egui::{Color32, Rect, Ui};

        struct SampleTab;

        impl TabInner for SampleTab {
            fn ui(&mut self, ui: &mut Ui, style: &Style, global: &mut Global) {
                let rect = ui.available_rect_before_wrap();

                let sep = rect.min.y + 25.0;

                let body_top = rect.intersect(Rect::everything_above(sep));
                ui.painter().rect_filled(body_top, 0.0, style.tab_base);

                let body = rect.intersect(Rect::everything_below(sep));
                ui.painter()
                    .rect_filled(body, 0.0, Color32::from_gray(0x28));
            }
        }

        struct NodeTodo;

        impl TabInner for NodeTodo {
            fn ui(&mut self, ui: &mut Ui, style: &Style, global: &mut Global) {
                let rect = ui.available_rect_before_wrap();
                let bg = Color32::from_gray(0x28);
                ui.painter().rect_filled(rect, 0.0, bg);
                crate::nodes::nodes(ui);
            }
        }

        let node_tree = NodeTree::new(&device, &window, surface_format, sample_count);
        let node_tree = Tab::new(crate::blender::NODETREE, "Node Tree", node_tree);
        let scene = Tab::new(crate::blender::VIEW3D, "Scene", SampleTab);
        let material = Tab::new(crate::blender::MATERIAL, "Material", NodeTodo);

        let outliner = Tab::new(crate::blender::OUTLINER, "Hierarchy", Hierarchy::default());
        let properties = Tab::new(
            crate::blender::PROPERTIES,
            "Inspector",
            Inspector::default(),
        );

        let root = TreeNode::leaf_with(vec![node_tree, scene, material]);
        let mut app = crate::app::App::new(&device, &window, surface_format, sample_count, root);

        let one = TreeNode::leaf_with(vec![properties]);
        let two = TreeNode::leaf_with(vec![outliner]);
        let [_, _b] = app.tree.split(NodeIndex::root(), one, Split::Right);
        let [_, _b] = app.tree.split(_b, two, Split::Above);

        app
    };

    let mut last_update_inst = Instant::now();
    let spawner = Spawner::new();

    event_loop.run_return(move |event, _, control_flow| {
        match event {
            Event::NewEvents(_) => (),
            Event::WindowEvent { event, .. } => match event {
                // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                // See: https://github.com/rust-windowing/winit/issues/208
                // This solves an issue where the app would panic when minimizing on Windows.
                WindowEvent::Resized(size) => {
                    if size.width > 0 && size.height > 0 {
                        fb.set_size(&device, size.width, size.height);
                    }
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                // Pass the winit events to the platform integration.
                event => app.on_event(event),
            },
            Event::DeviceEvent { .. } => (),
            Event::UserEvent(_) => window.request_redraw(),
            Event::Suspended | Event::Resumed | Event::MainEventsCleared => (),
            Event::RedrawRequested(..) => {
                let (target, frame) = match fb.next() {
                    Ok(frame) => frame,
                    // This error occurs when the app is minimized on Windows.
                    // Silently return here to prevent spamming the console with:
                    // "The underlying surface has changed, and therefore the swap chain must be updated"
                    Err(wgpu::SurfaceError::Outdated) => return,
                    Err(err) => return eprintln!("Dropped frame with error: {}", err),
                };

                let target = target.as_ref();

                let mut encoder = device.create_command_encoder(&Default::default());

                let repaint = app
                    .run(&window, &device, &queue, &mut encoder, target)
                    .unwrap();

                queue.submit(iter::once(encoder.finish()));
                frame.present();

                // Suppport reactive on windows only, but not on linux.
                *control_flow = if repaint {
                    ControlFlow::Poll
                } else {
                    ControlFlow::Wait
                };
            }

            Event::RedrawEventsCleared => {
                let target_frametime = Duration::from_secs_f64(1.0 / 60.0);
                let now = Instant::now();
                let time_since_last_frame = now - last_update_inst;
                if time_since_last_frame >= target_frametime {
                    window.request_redraw();
                    last_update_inst = now;
                } else {
                    *control_flow =
                        ControlFlow::WaitUntil(now + target_frametime - time_since_last_frame)
                };

                spawner.run_until_stalled();
            }

            Event::LoopDestroyed => (),
        }
    });
}

pub fn fuck_ref<'a, T>(ptr: &T) -> &'a T {
    unsafe { &*(ptr as *const T) }
}

pub fn fuck_mut<'a, T>(ptr: &mut T) -> &'a mut T {
    unsafe { &mut *(ptr as *mut T) }
}

// TODO: Stable since Rust version 1.62.0
pub fn total_cmp(lhs: &f32, rhs: &f32) -> std::cmp::Ordering {
    let mut lhs = lhs.to_bits() as i32;
    let mut rhs = rhs.to_bits() as i32;
    lhs ^= (((lhs >> 31) as u32) >> 1) as i32;
    rhs ^= (((rhs >> 31) as u32) >> 1) as i32;
    lhs.cmp(&rhs)
}

pub struct NodeTree {
    pub workspace: Workspace,
    pub context: egui::Context,
    pub state: egui_winit::State,
    pub renderpass: RenderPass,

    pub zoom_level: f32,

    pub raw_mouse_position: Option<egui::Pos2>,
}

impl NodeTree {
    pub fn new(
        device: &wgpu::Device,
        window: &Window,
        output_format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> Self {
        let limits = device.limits();
        let max_texture_side = limits.max_texture_dimension_2d as usize;

        Self {
            workspace: Workspace::default(),
            state: egui_winit::State::new(max_texture_side, window),
            renderpass: RenderPass::new(device, output_format, sample_count),
            context: {
                let context = egui::Context::default();
                context.set_fonts(app::fonts_with_blender());
                context
            },

            zoom_level: 1.0 / window.scale_factor() as f32,
            raw_mouse_position: None,
        }
    }

    fn adjust_zoom(&mut self, zoom_delta: f32, point: egui::Vec2) {
        if !self.context.wants_pointer_input() {
            let zoom_clamped = (self.zoom_level + zoom_delta).clamp(0.25, 2.0);
            let zoom_delta = zoom_clamped - self.zoom_level;
            self.zoom_level += zoom_delta;
            self.workspace.pan_offset += point * zoom_delta;
        }
    }
}

impl app::TabInner for NodeTree {
    fn ui(&mut self, ui: &mut egui::Ui, _style: &app::Style, global: &mut Global) {
        let rect = ui.available_rect_before_wrap();
        let bg = egui::Color32::from_gray(0x20);
        ui.painter().rect_filled(rect, 0.0, bg);
    }

    fn on_event(
        &mut self,
        event: &winit::event::WindowEvent<'static>,
        viewport: egui::Rect,
        parent_scale: f32,
    ) {
        use crate::app::panel::{rect_scale, viewport_relative_position};

        let mut event = event.clone();

        // Copy event so we can modify it locally
        let mouse_in_viewport = self
            .raw_mouse_position
            .map(|pos| rect_scale(viewport, parent_scale).contains(pos))
            .unwrap_or(false);

        #[allow(clippy::single_match)]
        match event {
            // Filter out scaling / resize events
            winit::event::WindowEvent::Resized(_)
            | winit::event::WindowEvent::ScaleFactorChanged { .. } => return,
            // Hijack mouse events so they are relative to the viewport and account for zoom level.
            winit::event::WindowEvent::CursorMoved {
                ref mut position, ..
            } => {
                self.raw_mouse_position =
                    Some(egui::Pos2::new(position.x as f32, position.y as f32));
                /*
                *position = viewport_relative_position_winit(
                    *position,
                    parent_scale,
                    viewport,
                    self.zoom_level,
                );
                    */
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } if mouse_in_viewport => {
                let mouse_pos = if let Some(raw_pos) = self.raw_mouse_position {
                    viewport_relative_position(raw_pos, parent_scale, viewport, 1.0).to_vec2()
                } else {
                    egui::Vec2::ZERO
                };
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, dy) => {
                        self.adjust_zoom(-dy as f32 * 8.0 * 0.01, mouse_pos)
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        self.adjust_zoom(-pos.y as f32 * 0.01, mouse_pos)
                    }
                }
            }

            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                ..
            } if !mouse_in_viewport => return,
            _ => {}
        }

        self.state.on_event(&self.context, &event);
    }

    fn render(&mut self, ctx: app::RenderContext) {
        use egui::*;
        use winit::dpi::PhysicalSize;
        use winit::event::WindowEvent;

        let app::RenderContext {
            device,
            queue,
            window,
            encoder,
            attachment,
            viewport,
        } = ctx;

        let parent_scale = window.scale_factor() as f32;

        crate::workspace::preview::update_previews(
            encoder,
            &mut self.workspace,
            device,
            &mut self.renderpass,
            parent_scale,
        );

        // We craft a fake resize event so that the code in egui_winit_platform
        // remains unchanged, thinking it lives in a real window. The poor thing!
        let fake_resize_event = WindowEvent::Resized(PhysicalSize::new(
            (viewport.width() * self.zoom_level * parent_scale) as u32,
            (viewport.height() * self.zoom_level * parent_scale) as u32,
        ));

        self.state.on_event(&self.context, &fake_resize_event);

        let mut new_input = self.state.take_egui_input(window);
        let ppi = self.zoom_level.recip();
        new_input.pixels_per_point = Some(ppi);
        {
            let viewport = crate::app::panel::rect_scale(viewport, ppi.recip());
            new_input.screen_rect = Some(viewport);
        }
        self.context.begin_frame(new_input);

        self.workspace.draw(&self.context);

        let FullOutput {
            shapes,
            needs_repaint: _,
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
            .add_textures(device, queue, &textures_delta)
            .unwrap();
        self.renderpass.remove_textures(textures_delta).unwrap();
        self.renderpass
            .update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui main render pass"),
            color_attachments: &[attachment],
            depth_stencil_attachment: None,
        });

        rpass.push_debug_group("egui_pass");

        self.renderpass
            .execute(&mut rpass, &paint_jobs, &screen_descriptor, viewport)
            .unwrap();

        rpass.pop_debug_group();

        //Ok(needs_repaint)
    }
}
