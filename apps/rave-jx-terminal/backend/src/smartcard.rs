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
    handle: Option<thread::JoinHandle<()>>,
    stop_tx: mpsc::Sender<()>,
}

impl Watcher {
    pub fn new() -> Self {
        let ctx = pcsc::Context::establish(pcsc::Scope::User).unwrap();
        let status = Arc::new(Mutex::new(Default::default()));

        let mut readers_buf = [0; 2048];
        let mut reader_states = vec![
            // Listen for reader insertions/removals, if supported.
            ReaderState::new(PNP_NOTIFICATION(), pcsc::State::UNAWARE),
        ];

        let (stop_tx, stop_rx) = mpsc::channel();

        // spawn a thread to watch for smartcard events
        let handle = thread::spawn({
            let status = status.clone();
            move || {
                'thread: loop {
                    if stop_rx.try_recv().is_ok() {
                        tracing::debug!("stopping smartcard watcher while listing readers");
                        break;
                    }

                    // Remove dead readers.
                    fn is_dead(rs: &ReaderState) -> bool {
                        rs.event_state()
                            .intersects(pcsc::State::UNKNOWN | pcsc::State::IGNORE)
                    }
                    reader_states.retain(|rs| !is_dead(rs));

                    // Add new readers.
                    let names = ctx
                        .list_readers(&mut readers_buf)
                        .expect("failed to list readers");
                    for name in names {
                        if !reader_states.iter().any(|rs| rs.name() == name) {
                            reader_states.push(ReaderState::new(name, pcsc::State::UNAWARE));
                        }
                    }

                    // Update the view of the state to wait on.
                    for rs in &mut reader_states {
                        rs.sync_current_state();
                    }

                    // Wait until the state changes.
                    'status_change: loop {
                        if stop_rx.try_recv().is_ok() {
                            tracing::debug!(
                                "stopping smartcard watcher while waiting for status change"
                            );
                            break 'thread;
                        }

                        match ctx.get_status_change(Duration::from_millis(50), &mut reader_states) {
                            Ok(()) => break 'status_change,
                            Err(pcsc::Error::Timeout) => {
                                // no change, keep waiting
                            }
                            Err(err) => {
                                tracing::error!("error getting status change: {:?}", err);
                                break 'status_change;
                            }
                        }
                    }

                    // Print current state.
                    *status.lock().unwrap() = SmartcardStatus::NoReader;

                    for rs in reader_states.iter() {
                        if rs.name() == PNP_NOTIFICATION() || is_dead(rs) {
                            continue;
                        }

                        if rs.event_state().intersects(pcsc::State::PRESENT) {
                            // found a card, break out of the loop
                            *status.lock().unwrap() = SmartcardStatus::Card;
                            break;
                        } else {
                            // found a reader but no card, keep looking
                            *status.lock().unwrap() = SmartcardStatus::NoCard;
                        }
                    }
                }
            }
        });

        Self {
            status,
            handle: Some(handle),
            stop_tx,
        }
    }

    pub fn status_getter(&self) -> StatusGetter {
        StatusGetter::new(self.status.clone())
    }

    pub fn stop(self) {
        // let `self` drop
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop_tx.send(()).unwrap();
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
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
