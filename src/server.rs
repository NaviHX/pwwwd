use common::ipc;
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{
    net::{UnixListener, UnixStream, unix::SocketAddr},
    select,
    sync::oneshot,
    task::JoinHandle,
};
use tracing::{error, info, warn};

pub struct Server {
    uds_listener: UnixListener,
    busy_flag: Arc<AtomicBool>,

    stop_signal_rx: oneshot::Receiver<()>,
}

impl Server {
    pub fn new(uds_addr: &str) -> Result<(Self, ServerHandle), io::Error> {
        let uds_listener = UnixListener::bind(uds_addr)?;
        let busy_flag = Arc::new(AtomicBool::new(false));
        let (stop_signal_tx, stop_signal_rx) = oneshot::channel();
        let server_handle = ServerHandle::new(stop_signal_tx);

        Ok((
            Self {
                uds_listener,
                busy_flag,
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
}

struct TaskHandle<T, E: Default> {
    busy_flag: Option<Arc<AtomicBool>>,
    signal: Option<oneshot::Sender<Result<T, E>>>,
}

const DROP_UNRELEASED_TASK_HANDLE: &str = "The task handle will be dropped,\
    but `signal` and `busy_flag` is not released!";
const ERR_SEND: &str = "Encountered error when send back signal to UDS server!\
    Maybe the receiver has already been droped!";
const ERR_TOGGLE_BUSY: &str = "Trying to release the busy_flag,\
    but it has already been released!";

impl<T, E: Default> Drop for TaskHandle<T, E> {
    fn drop(&mut self) {
        if self.signal.is_some() || self.busy_flag.is_some() {
            #[cfg(not(feature = "panic-dropping-unreleased-task-handle"))]
            warn!(DROP_UNRELEASED_TASK_HANDLE);
            #[cfg(feature = "panic-dropping-unreleased-task-handle")]
            panic!(DROP_UNRELEASED_TASK_HANDLE);
        }

        if let Some(signal) = self.signal.take() {
            match signal.send(Err(E::default())) {
                Ok(()) => {}
                Err(_) => error!(ERR_SEND),
            }
        }

        if let Some(busy_flag) = self.busy_flag.take() {
            match busy_flag.compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => {}
                Err(_) => error!(ERR_TOGGLE_BUSY),
            }
        }
    }
}

impl<T, E: Default> TaskHandle<T, E> {
    pub fn new(
        busy_flag: Option<Arc<AtomicBool>>,
        signal: Option<oneshot::Sender<Result<T, E>>>,
    ) -> Self {
        Self { busy_flag, signal }
    }

    pub fn succ(mut self, t: T) {
        match self.signal.take() {
            Some(signal) => match signal.send(Ok(t)) {
                Ok(()) => {}
                Err(_) => error!(ERR_SEND),
            },
            None => {}
        }

        if let Some(busy_flag) = self.busy_flag.take() {
            match busy_flag.compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => {}
                Err(_) => error!(ERR_TOGGLE_BUSY),
            }
        }
    }

    pub fn fail(mut self, e: E) {
        match self.signal.take() {
            Some(signal) => match signal.send(Err(e)) {
                Ok(()) => {}
                Err(_) => error!(ERR_SEND),
            },
            None => {}
        }

        if let Some(busy_flag) = self.busy_flag.take() {
            match busy_flag.compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => {}
                Err(_) => error!(ERR_TOGGLE_BUSY),
            }
        }
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
