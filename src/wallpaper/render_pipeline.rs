#[allow(clippy::too_many_arguments)]
pub fn create_pipeline(
    device: &wgpu::Device,
    layout_label: Option<&str>,
    pipeline_label: Option<&str>,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    shader: &wgpu::ShaderModule,

    vs_main: Option<&str>,
    buffers: &[wgpu::VertexBufferLayout],

    fs_main: Option<&str>,
    targets: &[Option<wgpu::ColorTargetState>],
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: layout_label,
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: pipeline_label,
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: vs_main,
            compilation_options: Default::default(),
            buffers,
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: fs_main,
            compilation_options: Default::default(),
            targets,
        }),
        multiview: None,
        cache: None,
    })
}
