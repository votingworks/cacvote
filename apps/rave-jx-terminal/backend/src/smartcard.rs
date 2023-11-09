use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use pcsc::{ReaderState, PNP_NOTIFICATION};
use types_rs::rave::jx::SmartcardStatus;

/// Watches for smartcard events.
pub fn watch() -> Watcher {
    Watcher::new()
}

/// Watches for smartcard events.
#[derive(Debug)]
pub struct Watcher {
    status: Arc<Mutex<SmartcardStatus>>,
    watcher: auth_rs::Watcher,
}

impl Watcher {
    pub fn new() -> Self {
        let watcher = auth_rs::Watcher::watch(None);

        thread::spawn(|| for event in watcher.events() {});

        Self {
            status: Arc::new(Mutex::new(SmartcardStatus::NoCard)),
            watcher,
        }
    }

    pub fn status_getter(&self) -> StatusGetter {
        StatusGetter::new(self.status.clone())
    }

    pub fn stop(self) {
        // let `self` drop
    }
}

/// Provides access to the current smartcard status.
#[derive(Debug, Clone)]
pub struct StatusGetter {
    status: Arc<Mutex<SmartcardStatus>>,
}

impl StatusGetter {
    fn new(status: Arc<Mutex<SmartcardStatus>>) -> Self {
        Self { status }
    }

    /// Gets the current smartcard status.
    pub fn get(&self) -> SmartcardStatus {
        self.status.lock().unwrap().clone()
    }
}
