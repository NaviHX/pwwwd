use super::super::vertex::Vertex;

pub const BUFFERS: &[wgpu::VertexBufferLayout] = &[Vertex::desc()];

pub fn target(format: wgpu::TextureFormat) -> Vec<Option<wgpu::ColorTargetState>> {
    vec![Some(wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    })]
}
