pub mod xfd;

use wgpu;

pub trait TransitionPass {
    fn render_pass(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        duration: f64,
        elapsed: f64,
        fps: f64,
        fill_color: (f64, f64, f64),
    );
}
