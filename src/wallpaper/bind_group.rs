pub mod texture_and_sampler {
    use wgpu;

    pub fn layout_desc(label: Option<&str>) -> wgpu::BindGroupLayoutDescriptor<'_> {
        wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        }
    }

    pub fn bind_group(
        device: &wgpu::Device,
        label: Option<&str>,
        layout: &wgpu::BindGroupLayout,
        texture_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        let desc = wgpu::BindGroupDescriptor {
            label,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        };

        device.create_bind_group(&desc)
    }

    pub fn layout_with_two_textures_desc(
        label: Option<&str>,
    ) -> wgpu::BindGroupLayoutDescriptor<'_> {
        wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        }
    }

    pub fn bind_group_with_two_textures(
        device: &wgpu::Device,
        label: Option<&str>,
        layout: &wgpu::BindGroupLayout,
        texture_1_view: &wgpu::TextureView,
        sampler_1: &wgpu::Sampler,
        texture_2_view: &wgpu::TextureView,
        sampler_2: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        let desc = wgpu::BindGroupDescriptor {
            label,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_1_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler_1),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(texture_2_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(sampler_2),
                },
            ],
        };

        device.create_bind_group(&desc)
    }
}

pub mod uniform {
    pub mod progress {
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct ProgressUniform(f32);

        use wgpu::util::DeviceExt;

        pub fn create_progress_buffer(device: &wgpu::Device, progress: f32) -> wgpu::Buffer {
            let progress = ProgressUniform(progress);
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Progress buffer"),
                contents: bytemuck::cast_slice(&[progress]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        }

        pub fn layout_desc(label: Option<&str>) -> wgpu::BindGroupLayoutDescriptor<'_> {
            wgpu::BindGroupLayoutDescriptor {
                label,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }
        }

        pub fn bind_group(
            device: &wgpu::Device,
            label: Option<&str>,
            layout: &wgpu::BindGroupLayout,
            progress_buffer: &wgpu::Buffer,
        ) -> wgpu::BindGroup {
            let desc = wgpu::BindGroupDescriptor {
                label,
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: progress_buffer.as_entire_binding(),
                }],
            };

            device.create_bind_group(&desc)
        }
    }
}
