use std::sync::Arc;

use pcsc::{ReaderState, State, PNP_NOTIFICATION};
use tokio::sync::mpsc;

use crate::{Event, SharedCardReaders};

pub struct Watcher {
    handle: Option<tokio::task::JoinHandle<()>>,
    stop_watch: mpsc::Sender<()>,
    receiver: mpsc::Receiver<Result<Event, pcsc::Error>>,
    card_readers: SharedCardReaders,
}

impl std::fmt::Debug for Watcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Watcher").finish()
    }
}

impl Watcher {
    /// Watch changes to card readers and cards, yielding information about each
    /// change to a mpsc channel.
    pub fn watch() -> Watcher {
        let (stop_watch_tx, mut stop_watch_rx) = mpsc::channel(1);
        let (tx, rx) = mpsc::channel(10);

        let mut readers_buf = [0; 2048];
        let mut reader_states = vec![
            // Listen for reader insertions/removals, if supported.
            ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE),
        ];

        let card_readers: SharedCardReaders = Arc::new(tokio::sync::Mutex::new(vec![]));

        let handle = tokio::spawn({
            let card_readers = card_readers.clone();
            async move {
                // each thread should have its own context
                let ctx = pcsc::Context::establish(pcsc::Scope::User).unwrap();

                'thread: loop {
                    if stop_watch_rx.try_recv().is_ok() {
                        break 'thread;
                    }

                    // Remove dead readers.
                    fn is_dead(rs: &ReaderState) -> bool {
                        rs.event_state().intersects(State::UNKNOWN | State::IGNORE)
                    }
                    for rs in &reader_states {
                        if is_dead(rs) {
                            let name = rs.name().to_str().unwrap().to_owned();
                            tx.send(Ok(Event::ReaderRemoved { reader_name: name }))
                                .await
                                .unwrap();
                        }
                    }
                    reader_states.retain(|rs| !is_dead(rs));

                    // Add new readers.
                    let names = match ctx.list_readers(&mut readers_buf) {
                        Ok(names) => names,
                        Err(err) => {
                            tx.send(Err(err)).await.unwrap();
                            continue;
                        }
                    };
                    for name in names {
                        if !reader_states.iter().any(|rs| rs.name() == name) {
                            reader_states.push(ReaderState::new(name, State::UNAWARE));
                            if tx
                                .send(Ok(Event::ReaderAdded {
                                    reader_name: name.to_str().unwrap().to_owned(),
                                }))
                                .await
                                .is_err()
                            {
                                break 'thread;
                            }
                        }
                    }

                    // Update the view of the state to wait on.
                    for rs in &mut reader_states {
                        rs.sync_current_state();
                    }

                    // Wait until the state changes.
                    'status_change: loop {
                        if stop_watch_rx.try_recv().is_ok() {
                            break 'thread;
                        }

                        let mut inner_reader_states = reader_states
                            .iter()
                            .map(|rs| ReaderState::new(rs.name().to_owned(), rs.event_state()))
                            .collect::<Vec<_>>();
                        let get_status_change_future = tokio::task::spawn_blocking({
                            let ctx = ctx.clone();
                            move || match ctx.get_status_change(None, &mut inner_reader_states) {
                                Ok(()) => Ok(inner_reader_states),
                                Err(err) => Err(err),
                            }
                        });

                        tokio::select! {
                            _ = stop_watch_rx.recv() => {
                                break 'thread;
                            }
                            result = get_status_change_future => {
                                match result.unwrap() {
                                    Ok(new_reader_states) => {
                                        reader_states = new_reader_states;
                                        break 'status_change;
                                    }
                                    Err(pcsc::Error::Timeout) => {
                                        // no change, keep waiting
                                    }
                                    Err(err) => {
                                        if tx.send(Err(err)).await.is_err() {
                                            break 'thread;
                                        }
                                        break 'status_change;
                                    }
                                }
                            }
                        };
                    }

                    for rs in &reader_states {
                        if rs.name() == PNP_NOTIFICATION() || is_dead(rs) {
                            continue;
                        }

                        if rs
                            .event_state()
                            .contains(pcsc::State::CHANGED | pcsc::State::PRESENT)
                        {
                            tracing::debug!("card inserted: {:?}", rs.name().to_str());
                            // found a card
                            let name = rs.name().to_str().unwrap().to_owned();
                            let reader_already_in_list = card_readers
                                .lock()
                                .await
                                .iter()
                                .any(|reader| **reader == name);

                            if !reader_already_in_list {
                                let mut card_readers = card_readers.lock().await;
                                card_readers.push(name.clone());
                            }
                            if tx
                                .send(Ok(Event::CardInserted {
                                    reader_name: name.clone(),
                                }))
                                .await
                                .is_err()
                            {
                                break 'thread;
                            }
                        }

                        if rs
                            .event_state()
                            .contains(pcsc::State::CHANGED | pcsc::State::EMPTY)
                        {
                            tracing::debug!("card removed: {:?}", rs.name().to_str());
                            // card removed
                            let name = rs.name().to_str().unwrap().to_owned();
                            let mut card_readers = card_readers.lock().await;
                            card_readers.retain(|r| *r != name);
                            if tx
                                .send(Ok(Event::CardRemoved { reader_name: name }))
                                .await
                                .is_err()
                            {
                                break 'thread;
                            }
                        }
                    }
                }

                tracing::debug!("exiting watcher loop");
            }
        });

        Watcher {
            handle: Some(handle),
            stop_watch: stop_watch_tx,
            receiver: rx,
            card_readers,
        }
    }

    pub async fn stop(&mut self) {
        self.stop_watch.send(()).await.unwrap();
        if let Some(handle) = self.handle.take() {
            handle.await.unwrap();
        }
    }

    pub async fn recv(&mut self) -> Option<Result<Event, pcsc::Error>> {
        self.receiver.recv().await
    }

    pub fn readers_with_cards(&self) -> SharedCardReaders {
        self.card_readers.clone()
    }
}
