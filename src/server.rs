use common::ipc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::{net::UnixListener, sync::oneshot};
use tracing::{error, warn};

pub struct Server {
    uds_listener: UnixListener,
    busy_flag: Arc<AtomicBool>,
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
            }
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
            }
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
