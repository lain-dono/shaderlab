pub struct Framebuffer {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    sample_count: u32,
    resolve_target: Option<wgpu::Texture>,
}

impl Framebuffer {
    pub fn new(
        device: &wgpu::Device,
        surface: wgpu::Surface,
        sample_count: u32,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        surface.configure(device, &config);

        Self {
            resolve_target: Self::multisampled(device, &config, sample_count),
            surface,
            config,
            sample_count,
        }
    }

    pub fn set_present_mode(&mut self, mode: wgpu::PresentMode) {
        self.config.present_mode = mode;
    }

    pub fn set_size(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(device, &self.config);
        self.set_sample_count(device, self.sample_count);
    }

    pub fn set_sample_count(&mut self, device: &wgpu::Device, sample_count: u32) {
        self.sample_count = sample_count;
        self.resolve_target = Self::multisampled(device, &self.config, self.sample_count);
    }

    pub fn next(&self) -> Result<(SurfaceTarget, wgpu::SurfaceTexture), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;

        Ok((
            SurfaceTarget {
                view: frame.texture.create_view(&Default::default()),
                target: self
                    .resolve_target
                    .as_ref()
                    .map(|t| t.create_view(&Default::default())),
                width: self.config.width,
                height: self.config.height,
            },
            frame,
        ))
    }

    fn multisampled(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Option<wgpu::Texture> {
        (sample_count > 1).then(|| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                dimension: wgpu::TextureDimension::D2,
                mip_level_count: 1,
                sample_count,
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                format: config.format,
            })
        })
    }
}

pub struct SurfaceTarget {
    pub view: wgpu::TextureView,
    pub target: Option<wgpu::TextureView>,

    pub width: u32,
    pub height: u32,
}

impl SurfaceTarget {
    pub fn as_ref(&self) -> Target {
        Target {
            view: &self.view,
            target: self.target.as_ref(),
            width: self.width,
            height: self.height,
        }
    }
}

pub struct Target<'a> {
    pub view: &'a wgpu::TextureView,
    pub target: Option<&'a wgpu::TextureView>,
    pub width: u32,
    pub height: u32,
}

impl<'a> Target<'a> {
    pub fn attach(
        &self,
        store: bool,
        clear: impl Into<Option<wgpu::Color>>,
    ) -> wgpu::RenderPassColorAttachment {
        let load = clear.into().map_or(wgpu::LoadOp::Load, wgpu::LoadOp::Clear);
        wgpu::RenderPassColorAttachment {
            view: self.target.unwrap_or(self.view),
            resolve_target: self.target.map(|_| self.view),
            ops: wgpu::Operations { load, store },
        }
    }
}
