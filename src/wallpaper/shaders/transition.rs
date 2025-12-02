pub mod xfd;

use common::cli::client::{TransitionKind, TransitionOptions};
use tracing::debug;
use wgpu;

use crate::wallpaper::shaders::transition::xfd::Xfd;

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
    }
}
