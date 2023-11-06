use std::sync::mpsc;

use pcsc::{ReaderState, State, PNP_NOTIFICATION};

#[derive(Debug)]
pub enum Event {
    ReaderAdded(String),
    ReaderRemoved(String),
}

pub struct Watcher {
    stop_watch: mpsc::Sender<()>,
    receiver: mpsc::Receiver<Result<Event, pcsc::Error>>,
}

impl Watcher {
    pub fn stop(&self) {
        self.stop_watch.send(()).unwrap();
    }

    pub fn receiver(&self) -> &mpsc::Receiver<Result<Event, pcsc::Error>> {
        &self.receiver
    }
}

pub struct CardReader {
    ctx: pcsc::Context,
}

impl CardReader {
    pub fn new() -> Result<Self, pcsc::Error> {
        // Establish a PC/SC context.
        let ctx = pcsc::Context::establish(pcsc::Scope::User)?;

        Ok(Self { ctx })
    }

    /// Watch changes to card readers and cards, yielding information about each
    /// change to a mpsc channel.
    pub fn watch(&mut self) -> Watcher {
        let ctx = self.ctx.clone();
        let (stop_watch_tx, stop_watch_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();

        let mut readers_buf = [0; 2048];
        let mut reader_states = vec![
            // Listen for reader insertions/removals, if supported.
            ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE),
        ];

        std::thread::spawn(move || {
            loop {
                if stop_watch_rx.try_recv().is_ok() {
                    break;
                }

                // Remove dead readers.
                fn is_dead(rs: &ReaderState) -> bool {
                    rs.event_state().intersects(State::UNKNOWN | State::IGNORE)
                }
                for rs in &reader_states {
                    if is_dead(rs) {
                        tx.send(Ok(Event::ReaderRemoved(
                            rs.name().to_str().unwrap().to_owned(),
                        )))
                        .unwrap();
                    }
                }
                reader_states.retain(|rs| !is_dead(rs));

                // Add new readers.
                let names = match ctx.list_readers(&mut readers_buf) {
                    Ok(names) => names,
                    Err(err) => {
                        tx.send(Err(err)).unwrap();
                        continue;
                    }
                };
                for name in names {
                    if !reader_states.iter().any(|rs| rs.name() == name) {
                        reader_states.push(ReaderState::new(name, State::UNAWARE));
                        tx.send(Ok(Event::ReaderAdded(name.to_str().unwrap().to_owned())))
                            .unwrap();
                    }
                }
            }
        });

        Watcher {
            stop_watch: stop_watch_tx,
            receiver: rx,
        }
    }

    // pub fn watch_card_status(&self) -> Result<(), Error> {
    //     // Establish a PC/SC context.
    //     let ctx = pcsc::Context::establish(pcsc::Scope::User)?;

    //     let mut readers_buf = [0; 2048];
    //     let mut reader_states = vec![ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE)];

    //     loop {
    //         // Remove dead readers.
    //         fn is_dead(rs: &ReaderState) -> bool {
    //             rs.event_state().intersects(State::UNKNOWN | State::IGNORE)
    //         }
    //         for rs in &reader_states {
    //             if is_dead(rs) {
    //                 println!("Removing {:?}", rs.name());
    //             }
    //         }
    //         reader_states.retain(|rs| !is_dead(rs));

    //         let names = ctx
    //             .list_readers(&mut readers_buf)
    //             .expect("failed to list readers");
    //         for name in names {
    //             if !reader_states.iter().any(|rs| rs.name() == name) {
    //                 println!("Adding {:?}", name);
    //                 reader_states.push(ReaderState::new(name, State::UNAWARE));

    //                 match ctx.connect(name, ShareMode::Exclusive, Protocols::ANY) {
    //                     Ok(card) => {
    //                         println!("Connected to card in reader {:?}", name);
    //                     }
    //                     Err(pcsc::Error::NoSmartcard) => {
    //                         println!("A smartcard is not present in the reader.");
    //                     }
    //                     Err(err) => {
    //                         eprintln!("Failed to connect to card in reader {:?}: {}", name, err);
    //                     }
    //                 }
    //             }
    //         }

    //         // Update the view of the state to wait on.
    //         for rs in &mut reader_states {
    //             rs.sync_current_state();
    //         }

    //         // Wait until the state changes.
    //         ctx.get_status_change(None, &mut reader_states)
    //             .expect("failed to get status change");

    //         // Print current state.
    //         println!();
    //         for rs in &reader_states {
    //             if rs.name() != PNP_NOTIFICATION() {
    //                 println!("{:?} {:?} {:?}", rs.name(), rs.event_state(), rs.atr());
    //             }
    //         }
    //     }

    //     Ok(())
    // }
}
