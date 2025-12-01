mod bind_group;
mod config;
mod misc;
mod off_screen;
mod render_pipeline;
mod sampler;
mod shaders;
mod texture;
mod vertex;

use anyhow::{Result, anyhow};
use common::cli::server as server_cli;
use config::Configurable;
use off_screen::OffScreen;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        },
    },
    shm::{Shm, ShmHandler},
};
use tracing::{debug, error, warn};
use wayland_client::{Connection, QueueHandle, globals::GlobalList};
use wgpu::{self, util::DeviceExt};

delegate_registry!(Wallpaper);
delegate_output!(Wallpaper);
delegate_compositor!(Wallpaper);
delegate_layer!(Wallpaper);
delegate_shm!(Wallpaper);

// TODO: Support sampler filter configuration in cli.
const MAG_FILTER: wgpu::FilterMode = wgpu::FilterMode::Linear;
const MIN_FILTER: wgpu::FilterMode = wgpu::FilterMode::Nearest;
const MIPMAP_FILTER: wgpu::FilterMode = wgpu::FilterMode::Nearest;

#[derive(Default)]
pub struct WallpaperBuilder {
    load_wallpaper: Option<String>,
    fill_color: Option<(f64, f64, f64)>,
    resize_option: Option<server_cli::ResizeOption>,

    mag_filter: Option<wgpu::FilterMode>,
    min_filter: Option<wgpu::FilterMode>,
    mipmap_filter: Option<wgpu::FilterMode>,
}

impl WallpaperBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_img_path(mut self, path: impl Into<String>) -> Self {
        self.load_wallpaper = Some(path.into());
        self
    }

    pub fn with_fill_color(mut self, rgb: (f64, f64, f64)) -> Self {
        self.fill_color = Some(rgb);
        self
    }

    pub fn with_resize_option(self, resize_option: server_cli::ResizeOption) -> Self {
        Self {
            resize_option: Some(resize_option),
            ..self
        }
    }

    pub fn with_mag_filter_mode(mut self, filter: wgpu::FilterMode) -> Self {
        self.mag_filter = Some(filter);
        self
    }

    pub fn with_min_filter_mode(mut self, filter: wgpu::FilterMode) -> Self {
        self.min_filter = Some(filter);
        self
    }

    pub fn with_mipmap_filter_mode(mut self, filter: wgpu::FilterMode) -> Self {
        self.mipmap_filter = Some(filter);
        self
    }

    #[tracing::instrument(skip(self, conn, globals, qh, namespace))]
    pub async fn build(
        self,
        conn: &Connection,
        globals: &GlobalList,
        qh: &QueueHandle<Wallpaper>,
        namespace: Option<impl Into<String>>,
    ) -> Result<Wallpaper> {
        let load_wallpaper = self
            .load_wallpaper
            .ok_or(anyhow!("No wallpaper provided"))?;

        let fill_color = self.fill_color.ok_or(anyhow!("No fill color provided"))?;

        debug!("Trying to create a wgpu instance ...");
        let instance = wgpu::Instance::new(&misc::instance_desc());

        debug!("Trying to prepare wayland event handlers ...");
        let registry_state = RegistryState::new(globals);
        let output_state = OutputState::new(globals, qh);
        let compositor_state = CompositorState::bind(globals, qh)?;
        let shm_state = Shm::bind(globals, qh)?;
        let layer_shell_state = LayerShell::bind(globals, qh)?;

        debug!("Trying to create a wayland layer surface");
        let orig_surface = compositor_state.create_surface(qh);
        let layer_surface = layer_shell_state.create_layer_surface(
            qh,
            orig_surface,
            Layer::Background,
            namespace,
            None,
        );

        debug!("Configuring the layer surface ...");
        // Ask the compositor to decide the size.
        layer_surface.set_size(0, 0);
        layer_surface.set_anchor(Anchor::all());
        // Ask the compositors not to move this surface to accommodate for other surfaces, and to
        // extend this surface all the way to the edges it anchored.
        layer_surface.set_exclusive_zone(-1);
        layer_surface.set_keyboard_interactivity(
            smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity::None,
        );
        // Do not forget to commit the surface, or we will never receive the first `configure`
        // event.
        layer_surface.commit();

        debug!("Trying to create a wgpu surface");
        let wgpu_surface = misc::layer_surface_to_wgpu_surface(conn, &layer_surface, &instance)?;

        debug!("Trying to request a wgpu adapter ...");
        let adapter = instance
            .request_adapter(&misc::adapter_options(&wgpu_surface))
            .await?;

        debug!("Trying to get a wgpu device and queue ...");
        let (device, queue) = adapter
            .request_device(&misc::device_desc(Some("pwwwd")))
            .await?;

        debug!("Trying to create a surface configuration ...");
        let surface_caps = wgpu_surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = config::create(&surface_format, &surface_caps);

        debug!("Trying to create the vertex buffer and index buffer ...");
        let resize_option = self
            .resize_option
            .ok_or(anyhow!("No resize option provided"))?;
        // HACK: As we don't know the surface size for now, use `stretch` to create the vertex buffer.
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

        let img = image::open(&load_wallpaper)?;
        let img = img.to_rgba8();
        let texture_width = img.width();
        let texture_height = img.height();
        let image_texture = {
            debug!("Trying to create and write to the texture ...");
            let size = texture::texture_size(texture_width, texture_height);
            let desc = texture::image_srgb_unorm_desc(None, size, 1);
            let texture = device.create_texture(&desc);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &img,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * texture_width),
                    rows_per_image: Some(texture_height),
                },
                size,
            );

            texture
        };

        debug!("Trying to create a sampler ...");
        let sampler = device.create_sampler(&sampler::desc(
            None,
            (
                wgpu::AddressMode::ClampToEdge,
                wgpu::AddressMode::ClampToEdge,
            ),
            self.mag_filter.unwrap_or(MAG_FILTER),
            self.min_filter.unwrap_or(MIN_FILTER),
            self.mipmap_filter.unwrap_or(MIPMAP_FILTER),
        ));

        debug!("Trying to bind the texture and the sampler together ...");
        let image_texture_view = image_texture.create_view(&texture::image_view_desc(None));
        let layout =
            device.create_bind_group_layout(&bind_group::texture_and_sampler::layout_desc(None));
        let bind_group = bind_group::texture_and_sampler::bind_group(
            &device,
            None,
            &layout,
            &image_texture_view,
            &sampler,
        );

        debug!("Trying to build the wallpaper shader ...");
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("wallpaper/shaders/wallpaper.wgsl"));

        debug!("Trying to build the wallpaper render pipeline ...");
        let render_pipeline = render_pipeline::create_pipeline(
            &device,
            None,
            Some("Render pipeline"),
            &[&layout],
            &shader,
            Some("vs_main"),
            shaders::wallpaper::BUFFERS,
            Some("fs_main"),

            // &shaders::wallpaper::targets(config.format),
            &shaders::wallpaper::targets(OffScreen::format()),
        );

        debug!("Creating off-screen buffer ...");
        // HACK: As we don't know the surface size for now, use `1920x1080` to create the
        // off-screen buffer.
        let off_screen_buffer = off_screen::OffScreen::create(&device, (1920, 1080), &config);

        debug!("Wallpaper built!");
        Ok(Wallpaper {
            layer_surface,
            exited: false,
            first_configured: false,
            damaged: true,

            off_screen_buffer,

            registry_state,
            output_state,
            compositor_state,
            shm_state,
            layer_shell_state,

            device,
            queue,
            wgpu_surface,
            config,

            image_texture,
            texture_width,
            texture_height,
            vertex_buffer,
            resize_option,
            index_buffer,
            fill_color,
            sampler,
            bind_group,
            render_pipeline,
        })
    }
}

pub struct Wallpaper {
    layer_surface: LayerSurface,

    // States
    /// Whether the daemon exited. If this flag is true, we should stop the event loop and exit.
    pub exited: bool,
    /// Whether the daemon finished the first configuration. If this flag is false, we cannot
    /// render to the surface and commit. Once `LayerShellHandler::configure` is called, this flag
    /// will be set to true.
    first_configured: bool,
    /// Whether we have something new to be drawed. `draw` method will render to the surface and
    /// commit if both `first_configured` and `damaged` are true. This flag will be set to true
    /// when:
    ///
    /// 1. A `new_size` is received by the daemon from the `LayerShellHandler::configure` method.
    /// 2. TODO: A new image path is received by the daemon from the client.
    /// 3. TODO: The daemon is doing transition work between two images.
    damaged: bool,

    /// Off-screen buffer
    off_screen_buffer: OffScreen,

    // Wayland event handlers,
    registry_state: RegistryState,
    output_state: OutputState,
    compositor_state: CompositorState,
    shm_state: Shm,
    layer_shell_state: LayerShell,

    // Image
    // image_rgba: RgbaImage,
    //
    // Because we will write the image into wgpu's texture later, we don't need to store the image
    // in struct's field.

    // Wgpu stuffs
    device: wgpu::Device,
    queue: wgpu::Queue,
    wgpu_surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    image_texture: wgpu::Texture,
    texture_width: u32,
    texture_height: u32,
    vertex_buffer: wgpu::Buffer,
    resize_option: server_cli::ResizeOption,
    index_buffer: wgpu::Buffer,
    fill_color: (f64, f64, f64),
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Wallpaper {
    #[tracing::instrument(skip(self, _conn, _qh))]
    pub fn draw(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>) {
        if !self.first_configured {
            warn!("The surface hasn't be configured yet. Stop drawing ...");
            return;
        }

        if !self.damaged {
            debug!("The surface has nothing new to draw. Stop drawing ...");
            return;
        }
        self.damaged = false;

        debug!("Damaging the whole surface ...");
        let width = self.config.width as i32;
        let height = self.config.height as i32;
        self.layer_surface.wl_surface().damage(0, 0, width, height);

        let output = match self.wgpu_surface.get_current_texture() {
            Ok(output) => output,
            Err(e) => {
                error!("Cannot get the current texture of the surface! : {e}");
                return;
            }
        };
        let view = output
            .texture
            .create_view(&texture::surface_view_desc(Some("Surface texture view")));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            // let (r, g, b) = self.fill_color;
            // let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            //     label: Some("Wallpaper render_pass"),
            //     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            //         view: &view,
            //         depth_slice: None,
            //         resolve_target: None,
            //         ops: wgpu::Operations {
            //             load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1.0 }),
            //             store: wgpu::StoreOp::Store,
            //         },
            //     })],
            //     depth_stencil_attachment: None,
            //     timestamp_writes: None,
            //     occlusion_query_set: None,
            // });
            //
            // render_pass.set_pipeline(&self.render_pipeline);
            // render_pass.set_bind_group(0, &self.bind_group, &[]);
            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // render_pass.draw_indexed(0..vertex::NUM_INDEX, 0, 0..1);

            self.off_screen_buffer.update_pass(
                &self.device,
                &mut encoder,
                (self.config.width, self.config.height),
                self.fill_color,
                &self.render_pipeline,
                &self.bind_group,
                &self.vertex_buffer,
                &self.index_buffer,
                vertex::NUM_INDEX,
            );
            self.off_screen_buffer
                .render_pass(&mut encoder, &view, self.fill_color);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    #[tracing::instrument(skip(self, conn, qh))]
    pub fn config(
        &mut self,
        configuration: LayerSurfaceConfigure,
        conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        self.config.with_new_size(configuration.new_size);
        self.wgpu_surface.configure(&self.device, &self.config);
        self.first_configured = true;

        // HACK: This the only way I know to get the surface size. Write to vertex buffer here.
        debug!("Writing into vertex buffer ...");
        let vertex_buffer = vertex::create_vertex_buffer_with_resize_option(
            (self.config.width, self.config.height),
            (self.texture_width, self.texture_height),
            self.resize_option,
        );
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertex_buffer));

        self.draw(conn, qh);
    }

    #[tracing::instrument(skip(self, qh))]
    pub async fn change_image_and_request_frame(
        &mut self,
        qh: &QueueHandle<Self>,
        image_path: &str,
        resize_option: server_cli::ResizeOption,
    ) -> Result<()> {
        // Load the new image.
        debug!("Trying to load the new image: {image_path}");
        let img = match image::open(image_path) {
            Ok(img) => img,
            Err(e) => {
                let report = format!("Failed to load the new image in `{image_path}`: {e}");
                error!("{}", report);
                return Err(anyhow!(report));
            }
        };
        let img = img.to_rgba8();
        let texture_width = img.width();
        let texture_height = img.height();
        let image_texture = {
            debug!("Trying to create and write to the texture ...");
            let size = texture::texture_size(texture_width, texture_height);
            let desc = texture::image_srgb_unorm_desc(None, size, 1);
            let texture = self.device.create_texture(&desc);
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &img,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * texture_width),
                    rows_per_image: Some(texture_height),
                },
                size,
            );

            texture
        };

        // Set the new texture;
        debug!("Set new texture for wallpaper ...");
        self.image_texture = image_texture;
        self.texture_width = texture_width;
        self.texture_height = texture_height;

        // Bind the new texture within the bind group.
        debug!("Trying to bind the new texture and the sampler together ...");
        let image_texture_view = self
            .image_texture
            .create_view(&texture::image_view_desc(None));
        let layout = self
            .device
            .create_bind_group_layout(&bind_group::texture_and_sampler::layout_desc(None));
        let bind_group = bind_group::texture_and_sampler::bind_group(
            &self.device,
            None,
            &layout,
            &image_texture_view,
            &self.sampler,
        );
        self.bind_group = bind_group;

        // Re-filling the vertex buffer.
        //
        // If we have the new resize option same as the old resize option, we can just skip the
        // building of the vertex buffer.
        if resize_option != self.resize_option {
            debug!("Re-filling the vertex buffer with the new resize option ...");
            let vertex_buffer = vertex::create_vertex_buffer_with_resize_option(
                (self.config.width, self.config.height),
                (self.texture_width, self.texture_height),
                self.resize_option,
            );
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertex_buffer));
        }

        // Request a new frame to draw the new wallpaper.
        self.damaged = true;
        let wl_surface = self.layer_surface.wl_surface().clone();
        self.layer_surface.wl_surface().frame(qh, wl_surface);
        self.layer_surface.commit();

        Ok(())
    }
}

impl ProvidesRegistryState for Wallpaper {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers!(OutputState);
}

impl OutputHandler for Wallpaper {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        debug!("`new_output` triggered");
    }

    fn update_output(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        debug!("`update_output` triggered");
    }

    fn output_destroyed(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _output: wayland_client::protocol::wl_output::WlOutput,
    ) {
        debug!("`output_destroyed` triggered");
    }
}

impl CompositorHandler for Wallpaper {
    fn scale_factor_changed(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        debug!("`scale_factor_changed` triggered");
    }

    fn transform_changed(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
        debug!("`transform_changed` triggered");
    }

    fn frame(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _time: u32,
    ) {
        debug!("`frame` triggered");
        self.draw(conn, qh);
    }

    fn surface_enter(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _output: &wayland_client::protocol::wl_output::WlOutput,
    ) {
        debug!("`surface_enter` triggered");
    }

    fn surface_leave(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _output: &wayland_client::protocol::wl_output::WlOutput,
    ) {
        debug!("`surface_leave` triggered")
    }
}

impl ShmHandler for Wallpaper {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}

impl LayerShellHandler for Wallpaper {
    fn closed(
        &mut self,
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _layer: &LayerSurface,
    ) {
        debug!("`closed` triggered. Exiting ...");
        self.exited = true;
    }

    fn configure(
        &mut self,
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
        layer: &LayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        debug!("`configure` triggered");

        if self.layer_surface != *layer {
            warn!("The layer doesn't match the surface stored in `Wallpaper`!");
            return;
        }

        self.damaged = true;
        self.config(configure, conn, qh);
        self.draw(conn, qh);
    }
}
