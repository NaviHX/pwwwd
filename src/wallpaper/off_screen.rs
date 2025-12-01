use crate::wallpaper::{
    MAG_FILTER, MIN_FILTER, MIPMAP_FILTER, bind_group, render_pipeline, sampler, shaders, texture,
    vertex,
};
use tracing::debug;
use wgpu::{self, util::DeviceExt};

pub struct OffScreen {
    frame: wgpu::Texture,

    // Rendering stuff
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
}

impl OffScreen {
    fn create_texture(device: &wgpu::Device, size: (u32, u32)) -> wgpu::Texture {
        let size = texture::texture_size(size.0, size.1);
        device.create_texture(&texture::offscreen_srgb_unorm_desc(
            Some("Off-screen texture"),
            size,
            1,
        ))
    }

    #[tracing::instrument(skip(self, device))]
    fn update_texture(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        debug!("Trying to create the new off-screen texture...");
        let size = texture::texture_size(size.0, size.1);
        let texture = device.create_texture(&texture::offscreen_srgb_unorm_desc(
            Some("Off-screen texture"),
            size,
            1,
        ));
        let texture_view = texture.create_view(&texture::image_view_desc(None));

        debug!("Trying to bind the new texture and the sampler together ...");
        let layout =
            device.create_bind_group_layout(&bind_group::texture_and_sampler::layout_desc(None));
        let bind_group = bind_group::texture_and_sampler::bind_group(
            &device,
            None,
            &layout,
            &texture_view,
            &self.sampler,
        );

        self.frame = texture;
        self.bind_group = bind_group;
    }

    /// Create a new empty off-screen rendering buffer.
    #[tracing::instrument(skip(device))]
    pub fn create(device: &wgpu::Device, size: (u32, u32), config: &wgpu::SurfaceConfiguration) -> Self {
        debug!("Creating off-screen texture ...");
        let texture = Self::create_texture(device, size);
        let texture_view = texture.create_view(&texture::surface_view_desc(None));

        debug!("Trying to build the off-screen shader ...");
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../wallpaper/shaders/wallpaper.wgsl"));

        debug!("Creating sampler ...");
        let sampler = device.create_sampler(&sampler::desc(
            None,
            (
                wgpu::AddressMode::ClampToEdge,
                wgpu::AddressMode::ClampToEdge,
            ),
            MAG_FILTER,
            MIN_FILTER,
            MIPMAP_FILTER,
        ));

        debug!("Trying to bind the texture and the sampler together ...");
        let layout =
            device.create_bind_group_layout(&bind_group::texture_and_sampler::layout_desc(None));
        let bind_group = bind_group::texture_and_sampler::bind_group(
            &device,
            None,
            &layout,
            &texture_view,
            &sampler,
        );

        // We will always draw the whole off-screen frame onto the entire surface, so it's fine for
        // us to use `STRETCH` resize option.
        debug!("Trying to create the vertex buffer and index buffer ...");
        let vertex_buffer = vertex::STRETCH_VERTICES;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(vertex_buffer),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = vertex::CCW_INDICES;
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(index_buffer),
            usage: wgpu::BufferUsages::INDEX,
        });

        debug!("Trying to build the off-screen render pipeline ...");
        let render_pipeline = render_pipeline::create_pipeline(
            &device,
            None,
            None,
            &[&layout],
            &shader,
            Some("vs_main"),
            shaders::wallpaper::BUFFERS,
            Some("fs_main"),
            &shaders::wallpaper::targets(config.format),
        );

        debug!("Off-screen buffer built");
        Self {
            frame: texture,
            render_pipeline,
            bind_group,
            sampler,
            vertices: vertex_buffer,
            indices: index_buffer,
            index_count: vertex::NUM_INDEX,
        }
    }

    /// Update the content of off-screen rendering buffer. Re-create the buffer if the new size
    /// doesn't equal to the current size.
    ///
    /// This `render_pass` will use `wallpaper.wgsl` shader.
    #[tracing::instrument(skip(
        self,
        device,
        encoder,
        render_pipeline,
        bind_group,
        vertex_buffer,
        index_buffer,
        index_buffer_len
    ))]
    pub fn update_pass(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        surface_size: (u32, u32),
        default_color: (f64, f64, f64),
        render_pipeline: &wgpu::RenderPipeline,
        bind_group: &wgpu::BindGroup,
        vertex_buffer: &wgpu::Buffer,
        index_buffer: &wgpu::Buffer,
        index_buffer_len: u32,
    ) {
        if surface_size != (self.frame.width(), self.frame.height()) {
            debug!("Re-creating the off-screen buffer to fit in the new size: {surface_size:?}");
            self.update_texture(device, surface_size);
        }

        debug!("Trying to render the off-screen buffer ...");
        let texture_view = self
            .frame
            .create_view(&texture::surface_view_desc(Some("Off-screen texture view")));

        let (r, g, b) = default_color;
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Off-screen render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
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

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..index_buffer_len, 0, 0..1);
    }

    /// Render to the surface. Just draw the whole off-screen buffer onto the entire surface.
    ///
    /// This render pass will use `wallpaper.wgsl` shader with `Stretch` resize option.
    #[tracing::instrument(skip(self, encoder, target_view))]
    pub fn render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        default_color: (f64, f64, f64),
    ) {
        debug!("Rendering the off-screen buffer to the surface ...");
        let (r, g, b) = default_color;
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Off-screen render pass"),
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
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }

    pub fn current_frame(&self) -> &wgpu::Texture {
        &self.frame
    }

    pub fn format() -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }
}
