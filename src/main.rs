mod server;
mod wallpaper;

use anyhow::Result;
use clap::Parser;
use common::cli::server as server_cli;
use tracing::{info, warn};
use wayland_client::{Connection, globals::registry_queue_init};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = server_cli::Args::parse();
    let mut builder = wallpaper::WallpaperBuilder::new();

    let image_path = match args.load {
        server_cli::Load::FromPath { path } => path,
        server_cli::Load::Restore { path } => {
            let restore_path = path.unwrap_or(server_cli::default_restore_path()?);
            let path = tokio::fs::read_to_string(restore_path).await?;
            path
        }
    };
    builder = builder.with_img_path(image_path);

    // TODO: implement resize mechanism.
    warn!(
        "Currently `pwwwd` doesn't `resize` options other than `stretch`.\
        All other options will fallback to `stretch`"
    );
    let resize = if args.resize.no_resize {
        server_cli::ResizeOption::No
    } else {
        args.resize.resize.unwrap_or(server_cli::DEFAULT_RESIZE)
    };
    builder = builder.with_resize_option(resize);

    let rgb_u8 = args.fill_rgb.unwrap_or(server_cli::RGB);
    let rgb_f64 = rgb_u8_to_f64(rgb_u8);
    builder = builder.with_fill_color(rgb_f64);

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();
    let mut wallpaper = builder
        .build(&conn, &globals, &qh, Option::<String>::None)
        .await?;

    loop {
        event_queue.blocking_dispatch(&mut wallpaper)?;

        if wallpaper.exited {
            break;
        }
    }

    info!("Exiting");
    Ok(())
}

fn rgb_u8_to_f64((r, g, b): (u8, u8, u8)) -> (f64, f64, f64) {
    (r as f64 / 255., g as f64 / 255., b as f64 / 255.)
}
