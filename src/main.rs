mod ease;
mod server;
mod wallpaper;

use anyhow::{Result, anyhow};
use clap::Parser;
use common::cli::{client::TransitionKind, server as server_cli};
use common::ipc;
use std::sync::Arc;
use tokio::{
    net::UnixStream,
    select,
    signal::unix::SignalKind,
    sync::{mpsc, oneshot},
};
use tracing::{debug, error, info, warn};
use wayland_client::QueueHandle;
use wayland_client::{Connection, globals::registry_queue_init};

use crate::{
    server::{Server, TaskHandle, TaskHub},
    wallpaper::Wallpaper,
};

const REQUSET_BUFFER_SIZE: usize = 4;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = server_cli::Args::parse();
    let mut builder = wallpaper::WallpaperBuilder::new();

    let image_path = match args.load {
        server_cli::Load::FromPath { path } => path,
        server_cli::Load::Restore => {
            let restore_path = server_cli::default_restore_path()?;
            let path = tokio::fs::read_to_string(restore_path).await?;
            tokio::fs::canonicalize(path).await?
        }
    };
    builder = builder.with_img_path(image_path);

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

    debug!("Trying to build the server ...");
    let (server, server_handle) = Server::new(common::ipc::default_uds_path()?)?;
    let task_hub = Arc::new(TaskHub::new());
    let (request_tx, mut request_rx) = mpsc::channel(REQUSET_BUFFER_SIZE);

    let server_join_handle = server.run(move |socket, addr| {
        let task_hub = task_hub.clone();
        let request_tx = request_tx.clone();

        async move {
            match task_hub.exclusively_exec(
                |task_handle, socket| async {
                    match process_connection(task_handle, socket, request_tx).await {
                        Ok(_) => debug!("Completed the task"),
                        Err(e) => error!("Failed to complete the task: {e}"),
                    }
                },
                socket,
            ) {
                Ok(fut) => fut.await,
                Err((e, mut socket)) => {
                    error!("{e}");
                    if let Err(e) = ipc::Reply::Error(format!("{e}"))
                        .async_send(&mut socket)
                        .await
                    {
                        error!("Failed to send reply back!: {e}");
                    }
                }
            }
        }
    });

    let mut shutdown_sig = wait_shutdown_sig().await?;

    loop {
        // Flush the outgoing buffers to ensure that the server does receive the messages we've
        // sent.
        event_queue.flush()?;

        // If other threads might be reading the wayland socket as well, make sure we don't have
        // any pending events.
        //
        // event_queue.dispatch_pending(&mut wallpapre)?;

        // Put in place some internal synchronization to prepare for the fact that we're going to
        // wait for events on the socket and read them.
        let read_guard = loop {
            match event_queue.prepare_read() {
                Some(g) => break g,
                None => {
                    event_queue.dispatch_pending(&mut wallpaper)?;
                }
            }
        };

        // Now we can wait for the wayland socket to be readable.
        //
        // When we come to handle events from other sources (e.g. messages sent by the client
        // through Unix domain socket), use the `select!` macro to wait for multiple sources.
        let fd = read_guard.connection_fd();
        let fd = tokio::io::unix::AsyncFd::new(fd)?;
        select! {
            _ = fd.readable() => {
                // `fd` borrows `read_guard`. To complete the read action, explicitly drop
                // `fd`.
                drop(fd);

                read_guard.read()?;
                event_queue.dispatch_pending(&mut wallpaper)?;
            }
            maybe_message = request_rx.recv() => {
                match maybe_message {
                    None => {
                        error!("All request senders have been dropped.\
                            It seems that the daemon server panicked!");
                        break;
                    }
                    Some((task_handle, message, reply_tx)) => {
                        if let ipc::Message::Kill = message {
                            info!("Received a shutdown signal from client, stopping ...");
                            if let Err(_) = reply_tx.send(ipc::Reply::Ok) {
                                error!("Failed to send shutdown reply to client!");
                            }

                            break
                        }

                        if let Err(e) = process_message(task_handle, message, reply_tx, &qh, &mut wallpaper).await {
                            error!("Failed to process request from client: {e}");
                        }
                    }
                }
            }
            maybe_signal = &mut shutdown_sig => {
                match maybe_signal {
                    Ok(_) => info!("Received a shutdown signal, stopping ..."),
                    Err(e) => error!("Failed to hook shutdown signal, stopping ... : {e}"),
                }

                break
            }
        }

        if wallpaper.exited {
            break;
        }
    }

    server_handle
        .stop()
        .map_err(|_| anyhow!("Server had stopped before the daemon exited"))?;
    server_join_handle.await?;
    info!("Exiting");

    Ok(())
}

fn rgb_u8_to_f64((r, g, b): (u8, u8, u8)) -> (f64, f64, f64) {
    (r as f64 / 255., g as f64 / 255., b as f64 / 255.)
}

async fn process_connection(
    task_handle: TaskHandle,
    mut socket: UnixStream,
    request_tx: mpsc::Sender<(TaskHandle, ipc::Message, oneshot::Sender<ipc::Reply>)>,
) -> Result<()> {
    let message = ipc::Message::async_receive(&mut socket).await?;
    let (reply_tx, reply_rx) = oneshot::channel();
    request_tx.send((task_handle, message, reply_tx)).await?;

    let reply_res = reply_rx.await;
    let reply = match reply_res {
        Ok(reply) => reply,
        Err(e) => ipc::Reply::Error(format!("Failed to get reply from event loop: {e}")),
    };
    reply.async_send(&mut socket).await?;

    Ok(())
}

async fn process_message(
    task_handle: TaskHandle,
    message: ipc::Message,
    reply_tx: oneshot::Sender<ipc::Reply>,
    qh: &QueueHandle<Wallpaper>,
    wallpaper: &mut Wallpaper,
) -> Result<()> {
    debug!("Message received: {message:?}");

    let reply = match message {
        ipc::Message::Kill => {
            error!("`Kill` request must be processed in outer scope");
            ipc::Reply::Ok
        }
        ipc::Message::Image { args } => {
            let image_path = args.path;
            let resize_option = args.resize;
            let transition_kind = args.transition;
            let transition_options = args.transition_options;

            if transition_kind != TransitionKind::No {
                info!("Starting transition: {image_path:?} ...");
                info!("Resize option: {resize_option:?}");
                info!("TransitionKind: {transition_kind:?}");

                let duration = transition_options
                    .duration
                    .unwrap_or(server_cli::DEFAULT_TRANSITION_DURATION);
                let fps = transition_options
                    .fps
                    .unwrap_or(server_cli::DEFAULT_TRANSITION_FPS);
                wallpaper
                    .start_transition(
                        &qh,
                        &image_path,
                        resize_option,
                        duration,
                        fps,
                        transition_kind,
                        transition_options,
                        task_handle,
                    )
                    .await;

                ipc::Reply::Ok
            } else {
                info!("Start immediate wallpaper switching: {image_path:?} ...");
                info!("Resize option: {resize_option:?}");
                let result = wallpaper
                    .change_image_and_request_frame(qh, &image_path, resize_option)
                    .await;

                ipc::Reply::from_result(result)
            }
        }
    };

    reply_tx
        .send(reply)
        .map_err(|_| anyhow!("Cannot send reply back to connection-processing task"))?;

    Ok(())
}

async fn wait_shutdown_sig() -> Result<oneshot::Receiver<()>> {
    debug!("Trying to hook stoppeing signal ...");
    let (sig_tx, sig_rx) = oneshot::channel();

    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())
        .map_err(|e| anyhow!("Failed to hook SIGINT: {e}"))?;
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())
        .map_err(|e| anyhow!("Failed to hook SIGTERM: {e}"))?;
    let mut sighup = tokio::signal::unix::signal(SignalKind::hangup())
        .map_err(|e| anyhow!("Failed to hook SIGHUP: {e}"))?;
    let mut sigquit = tokio::signal::unix::signal(SignalKind::quit())
        .map_err(|e| anyhow!("Failed to hook SIGQUIT: {e}"))?;

    tokio::spawn(async move {
        select! {
            _ = sigint.recv() => {},
            _ = sigterm.recv() => {},
            _ = sighup.recv() => {},
            _ = sigquit.recv() => {},
        };

        if let Err(_) = sig_tx.send(()) {
            error!("Failed to send stopping message from signal hooks!");
        }
    });

    Ok(sig_rx)
}
