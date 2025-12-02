use crate::{
    server::TaskHandle,
    wallpaper::{off_screen::OffScreen, shaders::transition::TransitionPass, texture},
};
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
    off_screen_buffer: OffScreen,
    first_rendered: bool,
    _task_handle: Option<TaskHandle>,
}

impl TransitionState {
    pub fn new(
        device: &wgpu::Device,
        start: Instant,
        duration: f64,
        fps: f64,
        transition: Box<dyn TransitionPass>,
        size: (u32, u32),
        target_format: wgpu::TextureFormat,
        task_handle: Option<TaskHandle>,
    ) -> Self {
        let off_screen_buffer = OffScreen::create(device, size, target_format);
        Self {
            start,
            duration,
            fps,
            last_rendered: start,
            transition,
            off_screen_buffer,
            first_rendered: false,
            _task_handle: task_handle,
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
        debug!("Start transition render pass ...");

        let elapsed_seconds = (now - self.start).as_secs_f64();
        debug!("elapsed_seconds={elapsed_seconds}");
        if elapsed_seconds > self.duration {
            debug!(
                "elapsed_seconds `{elapsed_seconds}` > duration `{}`",
                self.duration
            );
            return Err(TransitionRenderError::Finished);
        }

        let last_seconds = (self.last_rendered - self.start).as_secs_f64();
        let frame_duration = 1.0 / self.fps;
        debug!("last_seconds={last_seconds}, frame_duration={frame_duration}");

        let current_frame = (elapsed_seconds / frame_duration).floor() as i64;
        let last_frame = (last_seconds / frame_duration).floor() as i64;
        debug!("current_frame={current_frame}, last_frame={last_frame}");
        if current_frame == last_frame && self.first_rendered {
            debug!(
                "elapsed_seconds `{elapsed_seconds}` and \
                last `{last_seconds}` are in the same frame"
            );

            self.off_screen_buffer
                .render_pass(encoder, target_view, fill_color);
            return Err(TransitionRenderError::SameFrame);
        }

        debug!("New frame rendering");
        self.first_rendered |= true;
        self.last_rendered = now;
        let off_screen_view =
            self.off_screen_buffer
                .current_frame()
                .create_view(&texture::image_view_desc(Some(
                    "Transition off screen view",
                )));
        self.transition.render_pass(
            device,
            encoder,
            &off_screen_view,
            self.duration,
            elapsed_seconds,
            self.fps,
            fill_color,
        );
        self.off_screen_buffer
            .render_pass(encoder, target_view, fill_color);

        Ok(())
    }
}
