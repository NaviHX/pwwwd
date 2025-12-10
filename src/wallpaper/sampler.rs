pub fn desc(
    label: Option<&str>,
    (x_address_mode, y_address_mode): (wgpu::AddressMode, wgpu::AddressMode),
    mag_filter: wgpu::FilterMode,
    min_filter: wgpu::FilterMode,
    mipmap_filter: wgpu::FilterMode,
) -> wgpu::SamplerDescriptor<'_> {
    wgpu::SamplerDescriptor {
        label,
        address_mode_u: x_address_mode,
        address_mode_v: y_address_mode,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter,
        min_filter,
        mipmap_filter,
        ..Default::default()
    }
}
