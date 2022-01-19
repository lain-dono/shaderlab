use iced_wgpu::wgpu;
use iced_winit::Rectangle;

fn hash_shader(x: &str) -> u64 {
    use ahash::AHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = AHasher::default();
    x.hash(&mut hasher);
    hasher.finish()
}

#[derive(Default)]
pub struct Scene {
    pipeline: Option<wgpu::RenderPipeline>,
    hash: u64,
}

impl Scene {
    pub fn clear<'a>(
        &self,
        target: &'a wgpu::TextureView,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: crate::style::to_clear_color(crate::style::WORKSPACE_BG),
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
        if shader.is_empty() {
            return;
        }

        let hash = hash_shader(shader);
        if self.hash != hash {
            self.pipeline = build_pipeline(device, shader);
            self.hash = hash
        }

        if let Some(pipeline) = self.pipeline.as_ref() {
            let (x, y, w, h) = (viewport.x, viewport.y, viewport.width, viewport.height);
            render_pass.set_viewport(x.max(0.0), y.max(0.0), w.max(1.0), h.max(1.0), 0.0, 1.0);
            render_pass.set_pipeline(pipeline);
            render_pass.draw(0..3, 0..1);
        }
    }
}

fn build_pipeline(device: &wgpu::Device, shader: &str) -> Option<wgpu::RenderPipeline> {
    log::info!("wgsl:\n{}", shader);
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
            multiview: None,
        }),
    )
}
