use crate::wallpaper::shaders::transition::TransitionPass;
use std::time::Instant;
use thiserror::Error;
use tracing::debug;
use wgpu;

#[derive(Error, Debug)]
pub enum TransitionRenderError {
    #[error("Render too fast")]
    SameFrame,
    #[error("Transition finished")]
    Finished,
}

pub struct TransitionState {
    start: Instant,
    duration: f64,
    fps: f64,
    last_rendered: Instant,
    pub transition: Box<dyn TransitionPass>,
}

impl TransitionState {
    pub fn new(
        start: Instant,
        duration: f64,
        fps: f64,
        transition: Box<dyn TransitionPass>,
    ) -> Self {
        Self {
            start,
            duration,
            fps,
            last_rendered: start,
            transition,
        }
    }

    #[tracing::instrument(skip(self, device, encoder))]
    pub fn render_pass(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        now: Instant,
        target_view: &wgpu::TextureView,
        fill_color: (f64, f64, f64),
    ) -> Result<(), TransitionRenderError> {
        let elapsed_seconds = (now - self.start).as_secs_f64();
        if elapsed_seconds > self.duration {
            debug!(
                "elapsed_seconds `{elapsed_seconds}` > duration `{}`",
                self.duration
            );
            return Err(TransitionRenderError::Finished);
        }

        let last_seconds = (self.last_rendered - self.start).as_secs_f64();
        let frame_duration = 1.0 / self.fps;

        let current_frame = (elapsed_seconds / frame_duration).floor() as i64;
        let last_frame = (last_seconds / frame_duration).floor() as i64;
        if current_frame == last_frame {
            debug!(
                "elapsed_seconds `{elapsed_seconds}` and \
                last `{last_seconds}` are in the same frame"
            );
            return Err(TransitionRenderError::SameFrame);
        }

        self.last_rendered = now;
        self.transition.render_pass(
            device,
            encoder,
            target_view,
            self.duration,
            elapsed_seconds,
            self.fps,
            fill_color,
        );

        Ok(())
    }
}
