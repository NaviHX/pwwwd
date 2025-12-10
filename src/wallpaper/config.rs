pub fn create(
    format: &wgpu::TextureFormat,
    surface_caps: &wgpu::SurfaceCapabilities,
) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: *format,
        present_mode: surface_caps.present_modes[0],
        desired_maximum_frame_latency: 2,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],

        // Leave them to be one.
        // These fields will get valid values after the first configuration.
        width: 1,
        height: 1,
    }
}

pub trait Configurable {
    fn with_new_size(&mut self, size: (u32, u32)) -> &mut Self;
}

impl Configurable for wgpu::SurfaceConfiguration {
    fn with_new_size(&mut self, (width, height): (u32, u32)) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }
}
