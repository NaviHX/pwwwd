pub fn texture_size(width: u32, height: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    }
}

pub fn image_srgb_unorm_desc(
    label: Option<&str>,
    size: wgpu::Extent3d,
    sample_count: u32,
) -> wgpu::TextureDescriptor<'_> {
    wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    }
}

pub fn offscreen_srgb_unorm_desc(
    label: Option<&str>,
    size: wgpu::Extent3d,
    sample_count: u32,
) -> wgpu::TextureDescriptor<'_> {
    wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    }
}

pub fn image_view_desc(label: Option<&str>) -> wgpu::TextureViewDescriptor<'_> {
    wgpu::TextureViewDescriptor {
        label,
        ..Default::default()
    }
}

pub fn surface_view_desc(label: Option<&str>) -> wgpu::TextureViewDescriptor<'_> {
    wgpu::TextureViewDescriptor {
        label,
        ..Default::default()
    }
}
