use crate::wallpaper::{
    MAG_FILTER, MIN_FILTER, MIPMAP_FILTER, bind_group, render_pipeline, sampler,
    shaders::{self, transition::TransitionPass},
    vertex::{self, NUM_INDEX},
};
use wgpu::{self, util::DeviceExt};

#[allow(unused)]
pub struct Xfd {
    // Textures and samplers
    old_texture_view: wgpu::TextureView,
    new_texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,

    // Rendering stuffs
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    shader: wgpu::ShaderModule,
    render_pipeline: wgpu::RenderPipeline,

    // Progress stuffs
    progress_buffer: wgpu::Buffer,
    progress_bind_group_layout: wgpu::BindGroupLayout,
    progress_bind_group: wgpu::BindGroup,
}

impl Xfd {
    pub fn new(
        device: &wgpu::Device,
        old_texture_view: wgpu::TextureView,
        new_texture_view: wgpu::TextureView,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let sampler = device.create_sampler(&sampler::desc(
            Some("Xfd old sampler"),
            (
                wgpu::AddressMode::ClampToEdge,
                wgpu::AddressMode::ClampToEdge,
            ),
            MAG_FILTER,
            MIN_FILTER,
            MIPMAP_FILTER,
        ));

        let layout = device.create_bind_group_layout(
            &bind_group::texture_and_sampler::layout_with_two_textures_desc(Some(
                "Xfd bind group layout",
            )),
        );
        let bind_group = bind_group::texture_and_sampler::bind_group_with_two_textures(
            &device,
            Some("Xfd bind group"),
            &layout,
            &old_texture_view,
            &sampler,
            &new_texture_view,
            &sampler,
        );

        let progress_buffer = bind_group::uniform::progress::create_progress_buffer(device, 0.0);
        let progress_bind_group_layout = device.create_bind_group_layout(
            &bind_group::uniform::progress::layout_desc(Some("Progress layout")),
        );
        let progress_bind_group = bind_group::uniform::progress::bind_group(
            device,
            Some("Progress bind group"),
            &progress_bind_group_layout,
            &progress_buffer,
        );

        let vertex_buffer = vertex::STRETCH_VERTICES;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Xfd vertex buffer"),
            contents: bytemuck::cast_slice(&vertex_buffer),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = vertex::CCW_INDICES;
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Xfd vertex buffer"),
            contents: bytemuck::cast_slice(&index_buffer),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("xfd.wgsl"));
        let render_pipeline = render_pipeline::create_pipeline(
            device,
            Some("Xfd pipeline layout"),
            Some("Xfd pipeline"),
            &[&layout, &progress_bind_group_layout],
            &shader,
            Some("vs_main"),
            &shaders::wallpaper::BUFFERS,
            Some("fs_main"),
            &shaders::target(target_format),
        );

        Self {
            old_texture_view,
            new_texture_view,
            sampler,
            vertex_buffer,
            index_buffer,
            bind_group,
            shader,
            render_pipeline,
            progress_buffer,
            progress_bind_group_layout,
            progress_bind_group,
        }
    }
}

impl TransitionPass for Xfd {
    fn render_pass(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        progress: f32,
        fill_color: (f64, f64, f64),
    ) {
        self.progress_buffer =
            bind_group::uniform::progress::create_progress_buffer(device, progress);
        self.progress_bind_group = bind_group::uniform::progress::bind_group(
            device,
            Some("Progress"),
            &self.progress_bind_group_layout,
            &self.progress_buffer,
        );

        let (r, g, b) = fill_color;
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Xfd render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1.0 }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &self.progress_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..NUM_INDEX, 0, 0..1);
    }
}
