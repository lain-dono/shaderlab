use bevy::{
    core_pipeline::{draw_3d_graph, node, AlphaMask3d, Opaque3d, Transparent3d},
    ecs::query,
    prelude::*,
    render::{
        camera::{ActiveCamera, Camera, Camera3d, CameraTypePlugin, ExtractedCamera, RenderTarget},
        render_asset::RenderAssets,
        render_graph::{
            Node, NodeRunError, RenderGraph, RenderGraphContext, RenderGraphError, SlotValue,
        },
        render_phase::RenderPhase,
        render_resource::{std140::AsStd140, BindGroup, BindGroupLayout, Buffer, RenderPipeline},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{
            ExtractedView, ExtractedWindows, RenderLayers, ViewDepthTexture, ViewTarget,
            ViewUniform, ViewUniformOffset, ViewUniforms,
        },
        RenderApp, RenderStage,
    },
    window::WindowId,
};

// The name of the final node of the first pass.
pub const GIZMO_DRIVER: &str = "gizmo_driver";

#[derive(Default)]
pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        init_rendering(app.sub_app_mut(RenderApp)).unwrap();
    }
}

fn init_rendering(render_app: &mut App) -> Result<(), RenderGraphError> {
    // This will add 3D render phases for the new camera.
    //render_app.add_system_to_stage(RenderStage::Extract, extract_gizmo_camera_phases);

    let driver = GizmoCameraDriver::from_world(&mut render_app.world);

    let mut graph = render_app.world.resource_mut::<RenderGraph>();
    graph.add_node(GIZMO_DRIVER, driver);

    // CLEAR_PASS_DRIVER -> GIZMO_DRIVER -> MAIN_PASS_DRIVER
    //graph.add_node_edge(node::MAIN_PASS_DEPENDENCIES, GIZMO_DRIVER)?;
    //graph.add_node_edge(node::CLEAR_PASS_DRIVER, GIZMO_DRIVER)?;
    //graph.add_node_edge(GIZMO_DRIVER, node::MAIN_PASS_DRIVER)?;

    //graph.add_node_edge(GIZMO_DRIVER, node::MAIN_PASS_DEPENDENCIES)?;
    //graph.add_node_edge(GIZMO_DRIVER, node::MAIN_PASS_DRIVER)?;
    graph.add_node_edge(node::MAIN_PASS_DRIVER, GIZMO_DRIVER)?;

    Ok(())
}

// A node for the first pass camera that runs draw_3d_graph with this camera.
struct GizmoCameraDriver {
    camera: QueryState<
        (
            Entity,
            CRef<ExtractedView>,
            CRef<ViewTarget>,
            CRef<ViewDepthTexture>,
            CRef<ViewUniformOffset>,
        ),
        With<Camera3d>,
    >,
    lines: Lines,
    to_draw: u32,

    view_bind: Option<BindGroup>,
}

impl FromWorld for GizmoCameraDriver {
    fn from_world(world: &mut World) -> Self {
        let pipeline = LinesPipeline::from_world(world);

        let device = world.resource::<RenderDevice>();
        let lines = Lines::new(device);

        let view_uniforms = world.resource::<ViewUniforms>();
        let view_bind = view_uniforms.uniforms.binding().map(|resource| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lines buffer"),
                layout: &pipeline.camera_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource,
                }],
            })
        });

        world.insert_resource(pipeline);

        Self {
            camera: QueryState::new(world),
            lines,
            to_draw: 0,

            view_bind,
        }
    }
}

impl Node for GizmoCameraDriver {
    fn update(&mut self, world: &mut World) {
        self.camera.update_archetypes(world);

        /*
        let color = [[0x44; 4], [0x11; 4]];
        self.lines.grid_step_xz(Vec3::ZERO, 100, 4, 0.25, color);

        self.lines.cross(
            Vec3::ZERO,
            25.0,
            [self::clrs::X_AXIS, self::clrs::Y_AXIS, self::clrs::Z_AXIS],
        );
        */

        let pipeline = world.resource::<LinesPipeline>();
        let device = world.resource::<RenderDevice>();
        let queue = world.resource::<RenderQueue>();
        self.lines.vertex.upload(device, queue);
        self.to_draw = self.lines.index.upload(device, queue);

        let view_uniforms = world.resource::<ViewUniforms>();
        self.view_bind = view_uniforms.uniforms.binding().map(|resource| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lines buffer"),
                layout: &pipeline.camera_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource,
                }],
            })
        });
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<LinesPipeline>();

        if let Some(view_bind_group) = self.view_bind.as_ref() {
            for (camera, view, target, depth, offset) in self.camera.iter_manual(world) {
                //graph.run_sub_graph(draw_3d_graph::NAME, vec![SlotValue::Entity(camera)])?;

                let desc = wgpu::RenderPassDescriptor {
                    label: Some("lines render pass"),
                    color_attachments: &[target.get_color_attachment(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: false,
                        }),
                        stencil_ops: None,
                    }),
                };

                let mut render_pass = render_context.command_encoder.begin_render_pass(&desc);

                render_pass.set_bind_group(0, view_bind_group, &[offset.offset]);

                if self.to_draw > 0 {
                    render_pass.set_pipeline(&pipeline.pipeline_lines);
                    render_pass.set_vertex_buffer(0, *self.lines.vertex.buffer.slice(..));
                    render_pass.set_index_buffer(
                        *self.lines.index.buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    render_pass.draw_indexed(0..self.to_draw, 0, 0..1);
                }

                render_pass.set_pipeline(&pipeline.pipeline_grid);
                render_pass.draw(0..4, 0..1);
            }
        }

        Ok(())
    }
}

pub struct LinesPipeline {
    pub pipeline_lines: RenderPipeline,
    pub pipeline_grid: RenderPipeline,
    pub camera_layout: BindGroupLayout,
}

impl FromWorld for LinesPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.get_resource::<RenderDevice>().unwrap();

        //let shader_source = wgpu::include_wgsl!("gizmo.wgsl");
        //let shader_source = wgpu::include_wgsl!("gizmo.wgsl");
        let shader_source = std::fs::read_to_string("src/scene/gizmo.wgsl").unwrap();
        let shader_source = wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        };
        let shader_module = device.create_shader_module(&shader_source);

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gizmo camera bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(
                        wgpu::BufferSize::new(ViewUniform::std140_size_static() as u64).unwrap(),
                    ),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gizmo camera pipeline layout"),
            bind_group_layouts: &[&camera_layout],
            push_constant_ranges: &[],
        });

        let pipeline_lines = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lines render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main_line",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Unorm8x4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main_line",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::bevy_default(),
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                strip_index_format: Some(wgpu::IndexFormat::Uint32),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let pipeline_grid = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grid render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main_grid",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main_grid",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::bevy_default(),
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline_lines,
            pipeline_grid,
            camera_layout,
        }
    }
}

pub struct Lines {
    vertex: PodBuffer<Vertex>,
    index: PodBuffer<u32>,
}

impl Lines {
    pub fn new(device: &RenderDevice) -> Self {
        Self {
            vertex: PodBuffer::new(
                device,
                wgpu::BufferUsages::VERTEX,
                Some("lines vertices"),
                vec![],
            ),
            index: PodBuffer::new(
                device,
                wgpu::BufferUsages::INDEX,
                Some("lines indices"),
                vec![],
            ),
        }
    }

    pub fn extend_points(&mut self, color: [u8; 4], points: impl IntoIterator<Item = Vec3>) {
        let points = points.into_iter();
        self.extend(points.map(|position| Vertex { position, color }));
    }

    pub fn extend(&mut self, vertices: impl IntoIterator<Item = Vertex>) {
        let start = self.vertex.data.len() as u32;
        self.vertex.data.extend(vertices);
        let end = self.vertex.data.len() as u32;
        self.index.data.extend(start..end);
        self.index.data.push(0xFFFF_FFFF);
    }

    pub fn line(&mut self, start: Vec3, end: Vec3, color: [u8; 4]) {
        self.extend_points(color, [start, end]);
    }

    pub fn axis(&mut self, at: Vec3, size: f32, colors: [[u8; 4]; 3]) {
        let x = Vec3::new(size as f32, 0.0, 0.0);
        let y = Vec3::new(0.0, size as f32, 0.0);
        let z = Vec3::new(0.0, 0.0, size as f32);

        self.line(at, at + x, colors[0]);
        self.line(at, at + y, colors[1]);
        self.line(at, at + z, colors[2]);
    }

    pub fn cross(&mut self, at: Vec3, size: f32, colors: [[u8; 4]; 3]) {
        let x = Vec3::new(size as f32, 0.0, 0.0);
        let y = Vec3::new(0.0, size as f32, 0.0);
        let z = Vec3::new(0.0, 0.0, size as f32);

        self.line(at - x, at + x, colors[0]);
        self.line(at - y, at + y, colors[1]);
        self.line(at - z, at + z, colors[2]);
    }

    pub fn grid_step_xz(
        &mut self,
        at: Vec3,
        size: i32,
        step: i32,
        scale: f32,
        color: [[u8; 4]; 2],
    ) {
        let pos = size as f32;
        let neg = -size as f32;

        for i in -size..=size {
            let color = color[(i % step != 0) as usize];

            let iii = i as f32;
            self.line(
                at + Vec3::new(iii, 0.0, pos) * scale,
                at + Vec3::new(iii, 0.0, neg) * scale,
                color,
            );
            self.line(
                at + Vec3::new(pos, 0.0, iii) * scale,
                at + Vec3::new(neg, 0.0, iii) * scale,
                color,
            );
        }
    }
}

struct PodBuffer<T: bytemuck::Pod> {
    data: Vec<T>,
    buffer: Buffer,
    desc: wgpu::BufferDescriptor<'static>,
}

impl<T: bytemuck::Pod> PodBuffer<T> {
    fn new(
        device: &RenderDevice,
        usage: wgpu::BufferUsages,
        label: Option<&'static str>,
        data: Vec<T>,
    ) -> Self {
        let len = (std::mem::size_of::<T>() * data.len()) as wgpu::BufferAddress;

        let desc = wgpu::BufferDescriptor {
            label,
            size: len.next_power_of_two(),
            usage: wgpu::BufferUsages::COPY_DST | usage,
            mapped_at_creation: false,
        };

        let buffer = device.create_buffer(&desc);

        Self { buffer, data, desc }
    }

    fn upload(&mut self, device: &RenderDevice, queue: &RenderQueue) -> u32 {
        let len = (std::mem::size_of::<T>() * self.data.len()) as wgpu::BufferAddress;
        if len > self.desc.size {
            self.desc.size = len.next_power_of_two();
            self.buffer = device.create_buffer(&self.desc);
        }

        let data = bytemuck::cast_slice(&self.data);
        queue.write_buffer(&self.buffer, 0, data);

        let count = self.data.len() as u32;
        self.data.clear();
        count
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pub position: Vec3,
    pub color: [u8; 4],
}

impl Vertex {
    pub fn new(position: Vec3, color: [u8; 4]) -> Self {
        Self { position, color }
    }
}

pub struct CRef<T>(std::marker::PhantomData<T>);

impl<T: Component> query::WorldQuery for CRef<T> {
    type Fetch = query::ReadFetch<T>;
    type State = query::ReadState<T>;
    type ReadOnlyFetch = query::ReadFetch<T>;
}

pub struct CMut<T>(std::marker::PhantomData<T>);

impl<T: Component> query::WorldQuery for CMut<T> {
    type Fetch = query::WriteFetch<T>;
    type State = query::WriteState<T>;
    type ReadOnlyFetch = query::ReadOnlyWriteFetch<T>;
}

// see http://clrs.cc
pub mod clrs {
    #![allow(clippy::unreadable_literal, dead_code)]

    const fn rgb(c: u32) -> [u8; 4] {
        let [b, g, r, _] = c.to_le_bytes();
        [r, g, b, 0xFF]
    }

    pub const X_AXIS: [u8; 4] = RED;
    pub const Y_AXIS: [u8; 4] = GREEN;
    pub const Z_AXIS: [u8; 4] = BLUE;

    pub const XY_PLANE: [u8; 4] = Z_AXIS;
    pub const XZ_PLANE: [u8; 4] = Y_AXIS;
    pub const YZ_PLANE: [u8; 4] = X_AXIS;

    pub const GRID_COLOR: [u8; 4] = GRAY;

    pub const NAVY: [u8; 4] = rgb(0x001F3F);
    pub const BLUE: [u8; 4] = rgb(0x0074D9);
    pub const AQUA: [u8; 4] = rgb(0x7FDBFF);
    pub const TEAL: [u8; 4] = rgb(0x39CCCC);
    pub const OLIVE: [u8; 4] = rgb(0x3D9970);
    pub const GREEN: [u8; 4] = rgb(0x2ECC40);
    pub const LIME: [u8; 4] = rgb(0x01FF70);
    pub const YELLOW: [u8; 4] = rgb(0xFFDC00);
    pub const ORANGE: [u8; 4] = rgb(0xFF851B);
    pub const RED: [u8; 4] = rgb(0xFF4136);
    pub const MAROON: [u8; 4] = rgb(0x85144B);
    pub const FUCHSIA: [u8; 4] = rgb(0xF012BE);
    pub const PURPLE: [u8; 4] = rgb(0xB10DC9);
    pub const BLACK: [u8; 4] = rgb(0x111111);
    pub const GRAY: [u8; 4] = rgb(0xAAAAAA);
    pub const SILVER: [u8; 4] = rgb(0xDDDDDD);
    pub const WHITE: [u8; 4] = rgb(0xFFFFFF);
}
