use anyhow::{Result, anyhow};
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::shell::{WaylandSurface, wlr_layer::LayerSurface};
use std::ptr::NonNull;
use wayland_client::{Connection, Proxy};

pub fn instance_desc() -> wgpu::InstanceDescriptor {
    wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    }
}

pub fn layer_surface_to_wgpu_surface(
    conn: &Connection,
    layer_surface: &LayerSurface,
    instance: &wgpu::Instance,
) -> Result<wgpu::Surface<'static>> {
    let display = NonNull::new(conn.backend().display_ptr() as *mut _)
        .ok_or(anyhow!("Cannot get display handle from wayland connection"))?;
    let surface = NonNull::new(layer_surface.wl_surface().id().as_ptr() as *mut _)
        .ok_or(anyhow!("Cannot get window handle from wayland surface"))?;

    let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display));
    let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(surface));

    unsafe {
        Ok(
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })?,
        )
    }
}

pub fn adapter_options<'a>(
    surface: &'a wgpu::Surface<'static>,
) -> wgpu::RequestAdapterOptions<'a, 'static> {
    wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(surface),
    }
}

pub fn device_desc(label: Option<&str>) -> wgpu::DeviceDescriptor<'_> {
    wgpu::DeviceDescriptor {
        label,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::defaults(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: Default::default(),
        trace: wgpu::Trace::Off,
    }
}
