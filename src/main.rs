use futures::task::SpawnExt;
use iced_futures::Executor;
use iced_native::futures::{
    channel::mpsc,
    task::{Context, Poll},
    Sink,
};
use iced_native::{clipboard, command, window, Runtime};
use iced_wgpu::{wgpu, Backend, Renderer, Settings, Viewport};
use iced_winit::{conversion, futures, program, winit, Clipboard, Debug, Size};
use std::pin::Pin;
use winit::{
    dpi::PhysicalPosition,
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub mod controls;
pub mod graph;
pub mod node;
pub mod scene;
pub mod style;
pub mod widget;

use crate::{controls::Controls, scene::Scene};

pub fn main() {
    env_logger::init();

    // Initialize winit
    let event_loop: EventLoop<crate::controls::Message> = EventLoop::with_user_event();
    let proxy = event_loop.create_proxy();

    let mut runtime = {
        let proxy = Proxy::new(event_loop.create_proxy());
        let executor = iced_futures::executor::Smol::new().unwrap();

        Runtime::new(executor, proxy)
    };

    let window = winit::window::WindowBuilder::new()
        .with_title("WGSL ShaderLab")
        .with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080))
        .build(&event_loop)
        .unwrap();

    let physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
    let mut modifiers = ModifiersState::default();
    let mut clipboard = Clipboard::connect(&window);

    // Initialize wgpu
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let (device, queue) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Request adapter");

        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Request device")
    });

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;

    {
        let size = window.inner_size();

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        )
    }

    let mut resized = false;

    // Initialize staging belt and local pool
    let mut staging_belt = wgpu::util::StagingBelt::new(5 * 1024);
    let mut local_pool = futures::executor::LocalPool::new();

    // Initialize scene and GUI controls
    let mut scene = Scene::new(&device);
    let controls = Controls::new();

    // Initialize iced
    let mut debug = Debug::new();
    let mut renderer = Renderer::new(Backend::new(&device, Settings::default(), format));

    let mut state = program::State::new(
        controls,
        viewport.logical_size(),
        conversion::cursor_position(cursor_position, viewport.scale_factor()),
        &mut renderer,
        &mut debug,
    );

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        // You should change this if you want to render continuosly
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(_) => (),
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => cursor_position = position,
                    WindowEvent::ModifiersChanged(new_modifiers) => modifiers = new_modifiers,
                    WindowEvent::Resized(new_size) => {
                        viewport = Viewport::with_physical_size(
                            Size::new(new_size.width, new_size.height),
                            window.scale_factor(),
                        );

                        resized = true;
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                }

                // Map window event to iced event
                if let Some(event) =
                    iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers)
                {
                    state.queue_event(event);
                }
            }

            Event::DeviceEvent { .. } => (),
            Event::Suspended => (),
            Event::Resumed => (),

            Event::UserEvent(message) => state.queue_message(message),

            Event::MainEventsCleared => {
                // If there are events pending
                if !state.is_queue_empty() {
                    // We update iced
                    if let Some(command) = state.update(
                        viewport.logical_size(),
                        conversion::cursor_position(cursor_position, viewport.scale_factor()),
                        &mut renderer,
                        &mut clipboard,
                        &mut debug,
                    ) {
                        for action in command.actions() {
                            match action {
                                command::Action::Future(future) => runtime.spawn(future),

                                command::Action::Clipboard(action) => match action {
                                    clipboard::Action::Read(tag) => {
                                        let message = tag(clipboard.read());

                                        proxy
                                            .send_event(message)
                                            .expect("Send message to event loop");
                                    }
                                    clipboard::Action::Write(contents) => {
                                        clipboard.write(contents);
                                    }
                                },
                                command::Action::Window(action) => match action {
                                    window::Action::Resize { width, height } => {
                                        window.set_inner_size(winit::dpi::LogicalSize {
                                            width,
                                            height,
                                        });
                                    }
                                    window::Action::Move { x, y } => {
                                        window.set_outer_position(winit::dpi::LogicalPosition {
                                            x,
                                            y,
                                        });
                                    }
                                },
                            }
                        }
                    }

                    // and request a redraw
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if resized {
                    let size = window.inner_size();

                    surface.configure(
                        &device,
                        &wgpu::SurfaceConfiguration {
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            format,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::Mailbox,
                        },
                    );

                    resized = false;
                }

                let frame = if let Ok(frame) = surface.get_current_texture() {
                    frame
                } else {
                    println!("redraw");
                    window.request_redraw();
                    return;
                };

                let target = frame.texture.create_view(&Default::default());

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let program = state.program();

                {
                    // We clear the frame
                    let mut render_pass =
                        scene.clear(&target, &mut encoder, program.background_color());

                    // Draw the scene
                    scene.draw(
                        &device,
                        &mut render_pass,
                        program.workspace() * window.scale_factor() as f32,
                        program.source(),
                    );
                }

                // And then iced on top
                let mouse_interaction = renderer.backend_mut().draw(
                    &device,
                    &mut staging_belt,
                    &mut encoder,
                    &target,
                    &viewport,
                    state.primitive(),
                    &debug.overlay(),
                );

                // Then we submit the work
                staging_belt.finish();
                queue.submit(Some(encoder.finish()));
                frame.present();

                // Update the mouse cursor
                window
                    .set_cursor_icon(iced_winit::conversion::mouse_interaction(mouse_interaction));

                // And recall staging buffers
                local_pool
                    .spawner()
                    .spawn(staging_belt.recall())
                    .expect("Recall staging buffers");

                local_pool.run_until_stalled();
            }
            Event::RedrawEventsCleared => (),
            Event::LoopDestroyed => (),
            Event::PlatformSpecific(ps) => println!("{:?}", ps),
        }
    })
}

/// An event loop proxy that implements `Sink`.
#[derive(Clone, Debug)]
pub struct Proxy<Message: 'static> {
    raw: winit::event_loop::EventLoopProxy<Message>,
}

impl<Message: 'static> Proxy<Message> {
    /// Creates a new [`Proxy`] from an `EventLoopProxy`.
    pub fn new(raw: winit::event_loop::EventLoopProxy<Message>) -> Self {
        Self { raw }
    }
}

impl<Message: 'static> Sink<Message> for Proxy<Message> {
    type Error = mpsc::SendError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, message: Message) -> Result<(), Self::Error> {
        let _ = self.raw.send_event(message);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
