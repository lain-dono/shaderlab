use iced_wgpu::wgpu;
use iced_winit::{Color, Rectangle};

fn hash_shader(x: &str) -> u64 {
    use fnv::FnvBuildHasher;
    use std::hash::{BuildHasher, Hash, Hasher};
    let mut hasher = FnvBuildHasher::default().build_hasher();
    x.hash(&mut hasher);
    hasher.finish()
}

pub struct Scene {
    pipeline: wgpu::RenderPipeline,
    hash: u64,
}

impl Scene {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = "    return vec4<f32>(0.02, 0.02, 0.02, 1.0);\n";
        let pipeline = build_pipeline(device, &Self::wrap(shader)).unwrap();
        let hash = 0;
        Self { pipeline, hash }
    }

    pub fn clear<'a>(
        &self,
        target: &'a wgpu::TextureView,
        encoder: &'a mut wgpu::CommandEncoder,
        background_color: Color,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear({
                        let [r, g, b, a] = background_color.into_linear();

                        wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        })
    }

    pub fn draw<'a>(
        &'a mut self,
        device: &wgpu::Device,
        render_pass: &mut wgpu::RenderPass<'a>,
        viewport: Rectangle,
        shader: &str,
    ) {
        let hash = hash_shader(shader);
        if self.hash != hash_shader(shader) {
            if let Some(pipeline) = build_pipeline(device, &Self::wrap(shader)) {
                self.pipeline = pipeline;
            }
            self.hash = hash
        }

        let (x, y, w, h) = (viewport.x, viewport.y, viewport.width, viewport.height);
        render_pass.set_viewport(x.max(0.0), y.max(0.0), w.max(1.0), h.max(1.0), 0.0, 1.0);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..3, 0..1);
    }

    pub fn wrap(shader: &str) -> String {
        if shader.contains("[[stage(vertex)]]") {
            return shader.into();
        }
        /*
        let x = f32(i32(in_vertex_index) - 1);
        let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
        return vec4<f32>(x, y, 0.0, 1.0);
        */

        let _vs_triangle = r#"
            let x = i32(vertex_index) - 1;
            let y = (i32(vertex_index) & 1) * 2 - 1;
            return vec4<f32>(f32(x), f32(y), 0.0, 1.0);
        "#;
        let _vs_fullscreen = r#"
            let u = i32(vertex_index << 1u) & 2;
            let v = i32(vertex_index) & 2;
            return vec4<f32>(f32(u * 2 - 1), f32(-v * 2 + 1), 0.0, 1.0);
        "#;

        format!(
            r#"
[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> [[builtin(position)]] vec4<f32> {{
{}}}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] position: vec4<f32>) -> [[location(0)]] vec4<f32> {{
{}}}
"#,
            _vs_fullscreen, shader
        )
    }
}

fn build_pipeline(device: &wgpu::Device, shader: &str) -> Option<wgpu::RenderPipeline> {
    log::info!("wgsl: {}", shader);
    if let Err(err) = naga::front::wgsl::parse_str(shader) {
        err.emit_to_stderr(shader);
        return None;
    }

    let desc = wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(shader.into()),
    };

    let module = device.create_shader_module(&desc);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        push_constant_ranges: &[],
        bind_group_layouts: &[],
    });

    Some(
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }),
    )
}
