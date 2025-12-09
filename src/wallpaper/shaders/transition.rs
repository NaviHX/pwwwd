pub mod wipe;
pub mod xfd;

use common::cli::client::{DEFAULT_WIPE_ANGLE, TransitionKind, TransitionOptions};
use tracing::debug;
use wgpu;

use crate::wallpaper::shaders::transition::{wipe::Wipe, xfd::Xfd};

pub trait TransitionPass {
    fn render_pass(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        progress: f32,
        fill_color: (f64, f64, f64),
    );
}

#[tracing::instrument(skip(device, old_texture_view, new_texture_view))]
pub fn create_transition(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    old_texture_view: wgpu::TextureView,
    new_texture_view: wgpu::TextureView,
    transition_kind: TransitionKind,
    transition_options: TransitionOptions,
) -> Option<Box<dyn TransitionPass>> {
    match transition_kind {
        TransitionKind::No => None,

        TransitionKind::Xfd => {
            debug!("Xfd created");
            Some(Box::new(Xfd::new(
                device,
                old_texture_view,
                new_texture_view,
                target_format,
            )))
        }

        TransitionKind::Wipe => {
            debug!("Wipe created");
            Some(Box::new(Wipe::new(
                device,
                old_texture_view,
                new_texture_view,
                target_format,
                transition_options.wipe_angle.unwrap_or(DEFAULT_WIPE_ANGLE) as f32,
            )))
        }
    }
}

#[macro_export]
macro_rules! simple_transition_shader_code {
    ([ $old_texture:ident, $old_sampler:ident, $new_texture:ident, $new_sampler:ident ], $uniform_name:ident { $($data:ident : $data_type:ty => $uniform_field:ident : $uniform_type:ident),* }, | $fragment_shader_input_name:ident { $window_position:ident, $ndc_position:ident, $texture_coords:ident } | => { $fragment_shader:expr }) => {
        std::concat!(
            "
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position)", std::stringify!($window_position), ": vec4<f32>,
    @location(0)", std::stringify!($ndc_position), ": vec3<f32>,
    @location(1)", std::stringify!($texture_coords), ": vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.", std::stringify!($window_position), " = vec4<f32>(model.position, 1.0);
    out.", std::stringify!($ndc_position), " = model.position;
    out.", std::stringify!($texture_coords), " = model.texture_coords;
    return out;
}

@group(0) @binding(0)
var ", std::stringify!($old_texture), ": texture_2d<f32>;
@group(0) @binding(1)
var ", std::stringify!($old_sampler), ": sampler;
@group(0) @binding(2)
var ", std::stringify!($new_texture), ": texture_2d<f32>;
@group(0) @binding(3)
var ", std::stringify!($new_sampler), ": sampler;

struct Uniform {
    progress: f32,
    ", $(std::stringify!($uniform_field), ": ", std::stringify!($uniform_type), ",",)* "
}

@group(1) @binding(0)
var<uniform> ", std::stringify!($uniform_name), ": Uniform;

@fragment
fn fs_main(
    ", std::stringify!($fragment_shader_input_name), ": VertexOutput
) -> @location(0) vec4<f32> {",
    $fragment_shader, "
}"
        )
    };
}

#[macro_export]
macro_rules! simple_transition {
    ($name:ident, [ $old_texture:ident, $old_sampler:ident, $new_texture:ident, $new_sampler:ident ], $uniform_name:ident { $($data:ident : $data_type:ty => $uniform_field:ident : $uniform_type:ident),* }, | $fragment_shader_input_name:ident { $window_position:ident, $ndc_position:ident, $texture_coords:ident } | => { $fragment_shader:expr }) => {
        #[allow(unused)]
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct UniformData {
            progress: f32,
            $($data: $data_type,)*
        }

        #[allow(unused)]
        pub struct $name {
            old_texture_view: wgpu::TextureView,
            new_texture_view: wgpu::TextureView,
            sampler: wgpu::Sampler,

            vertex_buffer: wgpu::Buffer,
            index_buffer: wgpu::Buffer,
            bind_group: wgpu::BindGroup,
            shader: wgpu::ShaderModule,
            render_pipeline: wgpu::RenderPipeline,

            uniform_buffer: wgpu::Buffer,
            uniform_bind_group_layout: wgpu::BindGroupLayout,
            uniform_bind_group: wgpu::BindGroup,

            $($data: $data_type,)*
        }

        #[allow(unused)]
        impl $name {
            #[allow(unused)]
            pub fn new(
                device: &wgpu::Device,
                old_texture_view: wgpu::TextureView,
                new_texture_view: wgpu::TextureView,
                target_format: wgpu::TextureFormat,
                $($data: $data_type,)*
            ) -> Self {

                let sampler = device.create_sampler(&$crate::wallpaper::sampler::desc(
                    Some(std::concat!(std::stringify!($name), " sampler")),
                    (
                        wgpu::AddressMode::ClampToEdge,
                        wgpu::AddressMode::ClampToEdge,
                    ),
                    $crate::wallpaper::MAG_FILTER,
                    $crate::wallpaper::MIN_FILTER,
                    $crate::wallpaper::MIPMAP_FILTER,
                ));

                let layout = device.create_bind_group_layout(
                    &$crate::wallpaper::bind_group::texture_and_sampler::layout_with_two_textures_desc(Some(
                        std::concat!(std::stringify!($name), " bind group layout"),
                    )),
                );
                let bind_group = $crate::wallpaper::bind_group::texture_and_sampler::bind_group_with_two_textures(
                    &device,
                    Some(std::concat!(std::stringify!($name), " bindy group")),
                    &layout,
                    &old_texture_view,
                    &sampler,
                    &new_texture_view,
                    &sampler,
                );

                let uniform_buffer = $crate::wallpaper::bind_group::uniform::create_buffer(
                    device,
                    Some(std::concat!(std::stringify!($name), " data")),
                    UniformData {
                        progress: 0.0,
                        $($data,)*
                    },
                );
                let uniform_bind_group_layout =
                    device.create_bind_group_layout(&$crate::wallpaper::bind_group::uniform::layout_desc(
                        Some(std::concat!(std::stringify!($name), " data bind group layout")),
                        &$crate::wallpaper::bind_group::uniform::fragment_uniforms_layout_entries(1),
                    ));
                let uniform_bind_group = $crate::wallpaper::bind_group::uniform::progress::bind_group(
                    device,
                    Some(std::concat!(std::stringify!($name), " data bind group")),
                    &uniform_bind_group_layout,
                    &uniform_buffer,
                );

                let vertex_buffer = $crate::wallpaper::vertex::STRETCH_VERTICES;
                let vertex_buffer = wgpu::util::DeviceExt::create_buffer_init(device, &wgpu::util::BufferInitDescriptor {
                    label: Some(std::concat!(std::stringify!($name), " vertex buffer")),
                    contents: bytemuck::cast_slice(&vertex_buffer),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = $crate::wallpaper::vertex::CCW_INDICES;
                let index_buffer = wgpu::util::DeviceExt::create_buffer_init(device, &wgpu::util::BufferInitDescriptor {
                    label: Some(std::concat!(std::stringify!($name), " index buffer")),
                    contents: bytemuck::cast_slice(&index_buffer),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
                });

                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(std::concat!(std::stringify!($name), " shader module")),
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                        $crate::simple_transition_shader_code!(
                        [
                            $old_texture, $old_sampler,
                            $new_texture, $new_sampler
                        ],
                        $uniform_name {
                            $($data: $data_type => $uniform_field : $uniform_type),*
                        },
                        |$fragment_shader_input_name { $window_position, $ndc_position, $texture_coords }| => { $fragment_shader })))
                });
                let render_pipeline = $crate::wallpaper::render_pipeline::create_pipeline(
                    device,
                    Some(std::concat!(std::stringify!($name), " pipeline layout")),
                    Some(std::concat!(std::stringify!($name), " pipeline")),
                    &[&layout, &uniform_bind_group_layout],
                    &shader,
                    Some("vs_main"),
                    &$crate::wallpaper::shaders::wallpaper::BUFFERS,
                    Some("fs_main"),
                    &$crate::wallpaper::shaders::target(target_format),
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
                    uniform_buffer,
                    uniform_bind_group_layout,
                    uniform_bind_group,
                    $($data,)*
                }
            }
        }

        impl $crate::wallpaper::shaders::transition::TransitionPass for $name {
            #[allow(unused)]
            fn render_pass(
                &mut self,
                device: &wgpu::Device,
                encoder: &mut wgpu::CommandEncoder,
                target_view: &wgpu::TextureView,
                progress: f32,
                fill_color: (f64, f64, f64),
            ) {
                let data = UniformData {
                    progress,
                    $($data: self.$data,)*
                };
                self.uniform_buffer =
                    $crate::wallpaper::bind_group::uniform::create_buffer(device, Some(std::concat!(std::stringify!($name), " data")), data);
                self.uniform_bind_group = $crate::wallpaper::bind_group::uniform::uniforms_bind_group(
                    device,
                    Some("Progress"),
                    &self.uniform_bind_group_layout,
                    &[&self.uniform_buffer],
                );

                let (r, g, b) = fill_color;
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(std::concat!(std::stringify!($name), " render pass")),
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
                render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..$crate::wallpaper::vertex::NUM_INDEX, 0, 0..1);
            }
        }
    };
}
