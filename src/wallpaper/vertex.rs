use common::cli::server as server_cli;
use std::mem;
use wgpu;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    texture_coords: [f32; 2],
}

impl Vertex {
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

pub const NUM_INDEX: u32 = CCW_INDICES.len() as u32;
pub const CCW_INDICES: &[u16] = &[0, 1, 3, 1, 2, 3];

pub const STRETCH_VERTICES: &[Vertex] = &[
    Vertex {
        position: [1.0, 1.0, 0.0],
        texture_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-1., 1.0, 0.0],
        texture_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-1., -1., 0.0],
        texture_coords: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1., 0.0],
        texture_coords: [1.0, 1.0],
    },
];

macro_rules! vertices {
    ($px:expr, $nx:expr, $py:expr, $ny:expr, $wf:expr, $hf:expr) => {
        ::std::vec::Vec::from([
            Vertex { position: [$px, $py, 0.0], texture_coords: [$wf, 0.0] },
            Vertex { position: [$nx, $py, 0.0], texture_coords: [0.0, 0.0] },
            Vertex { position: [$nx, $ny, 0.0], texture_coords: [0.0, $hf] },
            Vertex { position: [$px, $ny, 0.0], texture_coords: [$wf, $hf] },
        ])
    }
}

pub fn create_vertex_buffer_with_resize_option(
    surface_size: (u32, u32),
    texture_size: (u32, u32),
    resize_option: server_cli::ResizeOption,
) -> Vec<Vertex> {
    let (sx, sy) = surface_size;
    let (tx, ty) = texture_size;

    match resize_option {
        server_cli::ResizeOption::No => {
            let (xf, yf) = (tx as f32 / sx as f32, ty as f32 / sy as f32);
            vertices!(xf, -xf, yf, -yf, 1.0, 1.0)
        }
        server_cli::ResizeOption::Crop => {
            let (xf, yf) = (tx as f32 / sx as f32, ty as f32 / sy as f32);
            if xf < yf {
                vertices!(1.0, -1., yf / xf, -yf / xf, 1.0, 1.0)
            } else {
                vertices!(xf / yf, -xf / yf, 1.0, -1., 1.0, 1.0)
            }
        }
        server_cli::ResizeOption::Fit => {
            let (xf, yf) = (tx as f32 / sx as f32, ty as f32 / sy as f32);
            if xf > yf {
                vertices!(1.0, -1., yf / xf, -yf / xf, 1.0, 1.0)
            } else {
                vertices!(xf / yf, -xf / yf, 1.0, -1., 1.0, 1.0)
            }
        }
        server_cli::ResizeOption::Stretch => vertices!(1.0, -1., 1.0, -1., 1.0, 1.0),
    }
}
