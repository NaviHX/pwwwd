use common::ipc;
use std::{
    io,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use thiserror::Error;
use tokio::{
    net::{UnixListener, UnixStream, unix::SocketAddr},
    select,
    sync::oneshot,
    task::JoinHandle,
};
use tracing::{error, info, warn};

pub struct Server {
    uds_listener: UnixListener,
    uds_path: PathBuf,

    stop_signal_rx: oneshot::Receiver<()>,
}

impl Server {
    pub fn new(uds_addr: impl AsRef<Path>) -> Result<(Self, ServerHandle), io::Error> {
        let uds_path = uds_addr.as_ref().to_owned();
        let uds_listener = UnixListener::bind(uds_addr)?;
        let (stop_signal_tx, stop_signal_rx) = oneshot::channel();
        let server_handle = ServerHandle::new(stop_signal_tx);

        Ok((
            Self {
                uds_listener,
                uds_path,
                stop_signal_rx,
            },
            server_handle,
        ))
    }

    pub fn run<FN, F>(mut self, handler: FN) -> JoinHandle<()>
    where
        FN: Send + Fn(UnixStream, SocketAddr) -> F + 'static,
        F: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(async move {
            info!("Server is running ...");

            loop {
                select! {
                    msg = &mut self.stop_signal_rx => {
                        match msg {
                            Ok(_) => info!("Server received stop signal. Stopping ..."),
                            Err(_) => warn!("Server handle dropped. Stopping ..."),
                        }

                        break
                    }

                    conn = self.uds_listener.accept() => match conn {
                        Ok((conn, addr)) => {
                            let task = handler(conn, addr);
                            tokio::spawn(task);
                        }
                        Err(e) => error!("Failed to accept a connection: {e}"),
                    }
                }
            }
        })
    }

    fn clean(&mut self) {
        info!("Do cleaning for the daemon server ...");

        if let Err(e) = std::fs::remove_file(&self.uds_path) {
            error!("Cannot remove the UDS path! : {e}");
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.clean();
    }
}

pub struct TaskHandle {
    busy_flag: Arc<AtomicBool>,
}

const ERR_TOGGLE_BUSY: &str = "Trying to release the busy_flag,\
    but it has already been released!";

impl TaskHandle {
    pub fn new(busy_flag: Arc<AtomicBool>) -> Self {
        Self { busy_flag }
    }

    fn finish(&mut self) {
        match self.busy_flag.compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => {}
            Err(_) => {
                #[cfg(not(feature = "panic-double-toggle-busy"))]
                error!(ERR_TOGGLE_BUSY);
                #[cfg(feature = "panic-double-toggle-busy")]
                panic!(ERR_TOGGLE_BUSY);
            }
        }
    }
}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        self.finish()
    }
}

pub struct TaskHub {
    busy_flag: Arc<AtomicBool>,
}

#[derive(Error, Debug)]
pub enum TaskHubError {
    #[error("The task hub is busy now")]
    Busy,
}

impl TaskHub {
    pub fn new() -> Self {
        Self {
            busy_flag: Arc::new(false.into()),
        }
    }

    fn create_handle(
        &self,
    ) -> Result<TaskHandle, TaskHubError> {
        if let Err(_) =
            self.busy_flag
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        {
            return Err(TaskHubError::Busy);
        }

        Ok(TaskHandle::new(self.busy_flag.clone()))
    }

    pub fn exclusively_exec<FN, F, ARG, T>(
        &self,
        f: FN,
        arg: ARG,
    ) -> Result<impl Future<Output = T> + Send + 'static, TaskHubError>
    where
        FN: Fn(TaskHandle, ARG) -> F + Send + 'static,
        F: Future<Output = T> + Send + 'static,
    {
        let handle = self.create_handle()?;
        let fut = f(handle, arg);
        Ok(fut)
    }
}

pub struct ServerHandle {
    stop_signal: oneshot::Sender<()>,
}

impl ServerHandle {
    pub fn new(stop_signal: oneshot::Sender<()>) -> Self {
        Self { stop_signal }
    }

    pub fn stop(self) -> Result<(), ()> {
        self.stop_signal.send(()).map_err(|_| ())
    }
}
