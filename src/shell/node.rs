use super::render::{
    EguiTexture, EguiTextureBindGroups, EguiTransform, EguiTransforms, ExtractedEguiContext,
    ExtractedEguiSettings, ExtractedRenderOutput, ExtractedWindowSizes,
};
use bevy::{
    core::cast_slice,
    ecs::world::{FromWorld, World},
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext},
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer, BufferUsages,
            ColorTargetState, ColorWrites, Extent3d, FrontFace, IndexFormat, LoadOp,
            MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, ShaderStages,
            ShaderType, TextureDimension, TextureFormat, TextureSampleType, TextureViewDimension,
            VertexStepMode,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::{BevyDefault, Image},
        view::ExtractedWindows,
    },
    window::WindowId,
};
use wgpu::{BufferDescriptor, SamplerBindingType};

pub struct EguiPipeline {
    pub pipeline: RenderPipeline,
    pub transform_bind_group_layout: BindGroupLayout,
    pub texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for EguiPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let shader_source = wgpu::include_wgsl!("shader.wgsl");
        let shader_module = render_device.create_shader_module(shader_source);

        let transform_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("egui transform bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(EguiTransform::min_size()),
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("egui texture bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("egui pipeline layout"),
            bind_group_layouts: &[&transform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline =
            render_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("egui render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: 20,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        blend: Some(BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::One,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                            alpha: BlendComponent {
                                src_factor: BlendFactor::One,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                        }),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    front_face: FrontFace::Cw,
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
            });

        Self {
            pipeline,
            transform_bind_group_layout,
            texture_bind_group_layout,
        }
    }
}

#[derive(Debug)]
struct Scissor {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl Scissor {
    fn new(rect: &egui::Rect, scale_factor: f32) -> Self {
        Self {
            x: (rect.min.x * scale_factor).round() as u32,
            y: (rect.min.y * scale_factor).round() as u32,
            w: (rect.width() * scale_factor).round() as u32,
            h: (rect.height() * scale_factor).round() as u32,
        }
    }

    fn validate(self, pw: u32, ph: u32) -> Option<Self> {
        if self.w > 0 && self.h > 0 && self.x < pw && self.y < ph {
            let Self { x, y, w, h } = self;

            let x_viewport_clamp = (x + w).saturating_sub(pw);
            let y_viewport_clamp = (y + h).saturating_sub(ph);
            let w = w.saturating_sub(x_viewport_clamp).max(1);
            let h = h.saturating_sub(y_viewport_clamp).max(1);

            Some(Self { x, y, w, h })
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct DrawCommand {
    vertices: u32,
    texture: EguiTexture,
    scissor: Scissor,
}

#[derive(Default)]
struct DataBuffer {
    data: Vec<u8>,
    capacity: usize,
    buffer: Option<Buffer>,
}

impl DataBuffer {
    fn update(&mut self, device: &RenderDevice, label: &str, usage: BufferUsages) {
        if self.data.len() > self.capacity {
            self.capacity = self.data.len().next_power_of_two();
            self.buffer = Some(device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size: self.capacity as wgpu::BufferAddress,
                usage: BufferUsages::COPY_DST | usage,
                mapped_at_creation: false,
            }));
        }
    }
}

pub struct EguiNode {
    window_id: WindowId,
    draw_commands: Vec<DrawCommand>,
    vertex: DataBuffer,
    index: DataBuffer,
}

impl EguiNode {
    pub fn new(window_id: WindowId) -> Self {
        Self {
            window_id,
            draw_commands: Vec::new(),
            vertex: DataBuffer::default(),
            index: DataBuffer::default(),
        }
    }
}

impl Node for EguiNode {
    fn update(&mut self, world: &mut World) {
        let shapes = {
            let mut shapes = world.get_resource_mut::<ExtractedRenderOutput>().unwrap();
            match shapes.0.get_mut(&self.window_id) {
                Some(shapes) => std::mem::take(&mut shapes.shapes),
                None => return,
            }
        };

        let window_size = &world.get_resource::<ExtractedWindowSizes>().unwrap().0[&self.window_id];
        let settings = &world.get_resource::<ExtractedEguiSettings>().unwrap().0;
        let device = world.get_resource::<RenderDevice>().unwrap();

        let scale_factor = window_size.scale_factor * settings.scale_factor as f32;
        if window_size.physical_width == 0 || window_size.physical_height == 0 {
            return;
        }

        let jobs = world.get_resource::<ExtractedEguiContext>().unwrap().0[&self.window_id]
            .tessellate(shapes);

        self.draw_commands.clear();
        self.vertex.data.clear();
        self.index.data.clear();

        let mut index_offset = 0;

        for egui::ClippedPrimitive {
            clip_rect,
            primitive,
        } in &jobs
        {
            let mesh = match primitive {
                egui::epaint::Primitive::Mesh(mesh) => mesh,
                egui::epaint::Primitive::Callback(_) => {
                    unimplemented!("Paint callbacks aren't supported")
                }
            };

            let (pw, ph) = (
                window_size.physical_width as u32,
                window_size.physical_height as u32,
            );

            let scissor = match Scissor::new(clip_rect, scale_factor).validate(pw, ph) {
                Some(scissor) => scissor,
                None => continue,
            };

            self.vertex
                .data
                .extend_from_slice(cast_slice(mesh.vertices.as_slice()));

            let indices_with_offset = mesh
                .indices
                .iter()
                .map(|i| i + index_offset)
                .collect::<Vec<_>>();

            self.index
                .data
                .extend_from_slice(cast_slice(indices_with_offset.as_slice()));

            index_offset += mesh.vertices.len() as u32;

            self.draw_commands.push(DrawCommand {
                vertices: mesh.indices.len() as u32,
                texture: match mesh.texture_id {
                    egui::TextureId::Managed(id) => EguiTexture::Managed(self.window_id, id),
                    egui::TextureId::User(id) => EguiTexture::User(id),
                },
                scissor,
            });
        }

        self.vertex
            .update(device, "egui vertex buffer", BufferUsages::VERTEX);
        self.index
            .update(device, "egui index buffer", BufferUsages::INDEX);
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.get_resource::<EguiPipeline>().unwrap();
        let render_queue = world.get_resource::<RenderQueue>().unwrap();

        let (vertex_buffer, index_buffer) = match (&self.vertex.buffer, &self.index.buffer) {
            (Some(vertex), Some(index)) => (vertex, index),
            _ => return Ok(()),
        };

        render_queue.write_buffer(vertex_buffer, 0, &self.vertex.data);
        render_queue.write_buffer(index_buffer, 0, &self.index.data);

        let bind_groups = &world
            .get_resource::<EguiTextureBindGroups>()
            .unwrap()
            .bind_groups;

        let transforms = world.get_resource::<EguiTransforms>().unwrap();

        let window = &world.get_resource::<ExtractedWindows>().unwrap().windows[&self.window_id];
        let view = window.swap_chain_texture.as_ref().unwrap();

        let desc = RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };

        let mut render_pass = render_context.command_encoder.begin_render_pass(&desc);

        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_vertex_buffer(0, *self.vertex.buffer.as_ref().unwrap().slice(..));
        render_pass.set_index_buffer(
            *self.index.buffer.as_ref().unwrap().slice(..),
            IndexFormat::Uint32,
        );

        let transform_buffer_offset = transforms.offsets[&self.window_id];
        let transform_buffer_bind_group = &transforms.bind_group.as_ref().unwrap().1;
        render_pass.set_bind_group(0, transform_buffer_bind_group, &[transform_buffer_offset]);

        let mut offset: u32 = 0;
        for cmd in &self.draw_commands {
            if cmd.scissor.x >= window.physical_width || cmd.scissor.y >= window.physical_height {
                continue;
            }

            let texture_bind_group = match bind_groups.get(&cmd.texture) {
                Some(texture_resource) => texture_resource,
                None => {
                    offset += cmd.vertices;
                    continue;
                }
            };

            render_pass.set_bind_group(1, texture_bind_group, &[]);

            render_pass.set_scissor_rect(
                cmd.scissor.x,
                cmd.scissor.y,
                cmd.scissor.w.min(window.physical_width),
                cmd.scissor.h.min(window.physical_height),
            );

            render_pass.draw_indexed(offset..(offset + cmd.vertices), 0, 0..1);
            offset += cmd.vertices;
        }

        Ok(())
    }
}

pub fn as_color_image(image: egui::ImageData) -> egui::ColorImage {
    match image {
        egui::ImageData::Color(image) => image,
        egui::ImageData::Font(image) => alpha_image_as_color_image(&image),
    }
}

pub fn alpha_image_as_color_image(image: &egui::FontImage) -> egui::ColorImage {
    let gamma = 1.0;
    egui::ColorImage {
        size: image.size,
        pixels: image.srgba_pixels(gamma).collect(),
    }
}

pub fn color_image_as_bevy_image(image: &egui::ColorImage) -> Image {
    let pixels = image
        .pixels
        .iter()
        .flat_map(|color| color.to_array())
        .collect();

    Image::new(
        Extent3d {
            width: image.width() as u32,
            height: image.height() as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
    )
}
