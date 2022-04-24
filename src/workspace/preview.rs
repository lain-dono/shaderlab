use crate::app::RenderPass;
use crate::builder::expr::*;
use crate::builder::FnBuilder;
use crate::builder::*;
use crate::nodes::master::expr_fullscreen;
use crate::workspace::{Node, Port, Storage};
use naga::{Binding, BuiltIn, EntryPoint, ShaderStage, Statement};

pub struct Preview {
    pub texture_view: wgpu::TextureView,
    pub texture_id: egui::TextureId,
    pub format: wgpu::TextureFormat,
    pub source: String,
    pub size: egui::Vec2,
    pub scale: f32,
}

impl Preview {
    pub fn new(
        device: &wgpu::Device,
        rpass: &mut RenderPass,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: (width as f32 * scale) as u32,
            height: (height as f32 * scale) as u32,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };

        let texture = device.create_texture(&desc);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_filter = wgpu::FilterMode::Nearest;

        Self {
            texture_id: rpass.egui_texture_from_wgpu_texture(device, &texture_view, texture_filter),
            texture_view,
            format,
            source: String::new(),
            size: egui::vec2(width as f32, height as f32),
            scale,
        }
    }

    pub fn pass<'a>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        clear: impl Into<Option<wgpu::Color>>,
    ) -> wgpu::RenderPass<'a> {
        let clear = clear.into();
        let load = clear.map(wgpu::LoadOp::Clear).unwrap_or(wgpu::LoadOp::Load);

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &self.texture_view,
                resolve_target: None,
                ops: wgpu::Operations { load, store: true },
            }],
            depth_stencil_attachment: None,
        })
    }
}

downcast_rs::impl_downcast!(PreviewBuilder);

pub trait PreviewBuilder: downcast_rs::Downcast {
    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }

    fn show_preview(&self) -> bool {
        true
    }

    fn ui(&mut self, _: &mut egui::Ui) {}

    fn output_expr(&self, _node: Node, _: &mut FnBuilder, _: Port) -> EmitResult;

    fn vertex(&self, _node: Node, function: &mut FnBuilder) -> EmitResult {
        expr_fullscreen(function)
    }

    fn fragment(&self, node: Node, function: &mut FnBuilder) -> EmitResult {
        if let Some(&port) = function.module.storage.nodes[node].outputs.first() {
            let expr = self.output_expr(node, function, port)?;
            function.resolve_to(expr, VectorKind::V4)
        } else {
            Err(EmitError::PortNotFound)
        }
    }

    fn pipeline(
        &self,
        node: Node,
        storage: &Storage,
        device: &wgpu::Device,
    ) -> EmitResult<(String, wgpu::RenderPipeline)> {
        let mut module = ModuleBuilder::from_wgsl(storage, include_str!("builtin.wgsl")).unwrap();

        let ty = BaseTypes::new(&mut module);

        let vs_input = StructBuilder::new(&mut module.module, "VertexInput")
            .builtin("vertex_index", ty.u32, BuiltIn::VertexIndex)
            .build();

        let vs_output = StructBuilder::new(&mut module.module, "VertexOutput")
            .builtin("vertex_position", ty.f32x4, BuiltIn::Position)
            .build();

        let fs_input = StructBuilder::new(&mut module.module, "FragmentInput")
            .builtin("builtin_position", ty.f32x4, BuiltIn::Position)
            .builtin("builtin_font_facing", ty.bool, BuiltIn::FrontFacing)
            .build();

        module.entry(|module| {
            let mut function = module.function();
            function.argument("input", vs_input, None);
            function.function.result = Some(naga::FunctionResult {
                ty: vs_output,
                binding: None,
            });

            let position = {
                let value = self.vertex(node, &mut function);
                if value.is_err() && !matches!(value, Err(EmitError::MaybeDefault)) {
                    dbg!(&value);
                }
                value?
            };

            let value = Some(function.emit(naga::Expression::Compose {
                ty: vs_output,
                components: vec![position],
            }));
            function.statement(Statement::Return { value });

            Ok(EntryPoint {
                name: String::from("vs_main"),
                stage: ShaderStage::Vertex,
                early_depth_test: None,
                workgroup_size: [0; 3],
                function: function.function,
            })
        })?;

        module.entry(|module| {
            let mut function = module.function();
            function.argument("input", fs_input, None);
            function.set_result(
                ty.f32x4,
                Binding::Location {
                    location: 0,
                    interpolation: Some(naga::Interpolation::Perspective),
                    sampling: Some(naga::Sampling::Center),
                },
            );

            let value = self.fragment(node, &mut function);
            if value.is_err() && !matches!(value, Err(EmitError::MaybeDefault)) {
                dbg!(&value);
            }
            let value = Some(value?);
            function.statement(Statement::Return { value });

            Ok(EntryPoint {
                name: String::from("fs_main"),
                stage: ShaderStage::Fragment,
                early_depth_test: None,
                workgroup_size: [0; 3],
                function: function.function,
            })
        })?;

        let source = module.build()?;

        let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.as_str().into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                targets: &[self.format().into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok((source, pipeline))
    }

    fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>, pipeline: &'a wgpu::RenderPipeline) {
        rpass.set_pipeline(pipeline);
        rpass.draw(0..3, 0..1);
    }
}

pub fn update_previews(
    encoder: &mut wgpu::CommandEncoder,
    workspace: &mut crate::workspace::Workspace,
    device: &wgpu::Device,
    rpass: &mut RenderPass,
    scale_factor: f32,
) {
    let storage = crate::fuck_ref(&workspace.storage);

    if !workspace.dirty {
        return;
    }
    workspace.dirty = false;

    for (node_key, node) in &mut workspace.storage.nodes {
        let builder = node.builder.as_ref();

        if !builder.show_preview() {
            continue;
        }

        // XXX: 2.0 const from ./node.rs
        let width_height = (node.rect.width() - 2.0).floor() as u32;

        if width_height == 0 {
            continue;
        }

        let mut preview = node.preview.take().unwrap_or_else(|| {
            Preview::new(
                device,
                rpass,
                builder.format(),
                width_height,
                width_height,
                scale_factor,
            )
        });

        if preview.scale != scale_factor
            || preview.size.x as u32 != width_height
            || preview.size.y as u32 != width_height
        {
            rpass
                .remove_textures(egui::TexturesDelta {
                    set: ahash::AHashMap::default(),
                    free: vec![preview.texture_id],
                })
                .unwrap();
            preview = Preview::new(
                device,
                rpass,
                builder.format(),
                width_height,
                width_height,
                scale_factor,
            );
        }

        let pipeline = builder.pipeline(node_key, storage, device);

        node.preview_is_valid = pipeline.is_ok();

        if let Ok((source, pipeline)) = pipeline {
            preview.source = source;

            let mut rpass = preview.pass(encoder, wgpu::Color::TRANSPARENT);
            builder.draw(&mut rpass, &pipeline);
        } else {
            let _ = preview.pass(encoder, wgpu::Color::TRANSPARENT);
        }

        node.preview = Some(preview);
    }
}
