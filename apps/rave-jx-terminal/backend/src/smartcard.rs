use std::sync::{Arc, Mutex};
use std::thread;

use pcsc::{ReaderState, PNP_NOTIFICATION};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum Status {
    NoReader,
    NoCard,
    Card,
}

pub(crate) fn watch() -> Watcher {
    Watcher::new()
}

pub(crate) struct Watcher {
    status: Arc<Mutex<Status>>,
}

impl Watcher {
    pub(crate) fn new() -> Self {
        let ctx = pcsc::Context::establish(pcsc::Scope::User).unwrap();
        let status = Arc::new(Mutex::new(Status::NoReader));

        let mut readers_buf = [0; 2048];
        let mut reader_states = vec![
            // Listen for reader insertions/removals, if supported.
            ReaderState::new(PNP_NOTIFICATION(), pcsc::State::UNAWARE),
        ];

        // spawn a thread to watch for smartcard events
        thread::spawn({
            let status = status.clone();
            move || {
                loop {
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
                    ctx.get_status_change(None, &mut reader_states)
                        .expect("failed to get status change");

                    // Print current state.
                    *status.lock().unwrap() = Status::NoReader;

                    for rs in reader_states.iter() {
                        if rs.name() == PNP_NOTIFICATION() || is_dead(rs) {
                            continue;
                        }

                        if rs.event_state().intersects(pcsc::State::PRESENT) {
                            // found a card, break out of the loop
                            *status.lock().unwrap() = Status::Card;
                            break;
                        } else {
                            // found a reader but no card, keep looking
                            *status.lock().unwrap() = Status::NoCard;
                        }
                    }
                }
            }
        });

        Self { status }
    }

    pub(crate) fn status_getter(&self) -> StatusGetter {
        StatusGetter::new(self.status.clone())
    }
}

#[derive(Clone)]
pub(crate) struct StatusGetter {
    status: Arc<Mutex<Status>>,
}

impl StatusGetter {
    pub(crate) fn new(status: Arc<Mutex<Status>>) -> Self {
        Self { status }
    }

    pub(crate) fn get(&self) -> Status {
        self.status.lock().unwrap().clone()
    }
}
