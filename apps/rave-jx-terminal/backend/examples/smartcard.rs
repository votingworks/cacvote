use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use pcsc::{Protocols, ReaderState, ShareMode, PNP_NOTIFICATION};

struct SmartcardWatcherController {
    running: Arc<Mutex<bool>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl SmartcardWatcherController {
    fn stop(mut self) {
        *self.running.lock().unwrap() = false;
        self.handle.take().unwrap().join().unwrap();
    }
}

struct SmartcardWatcher {
    ctx: pcsc::Context,
    connected: Arc<Mutex<bool>>,
}

impl SmartcardWatcher {
    fn new() -> Self {
        let ctx = pcsc::Context::establish(pcsc::Scope::User).unwrap();

        Self {
            ctx,
            connected: Arc::new(Mutex::new(false)),
        }
    }

    fn watch(&self) -> SmartcardWatcherController {
        let ctx = self.ctx.clone();
        let running = Arc::new(Mutex::new(true));
        let connected = self.connected.clone();

        let mut readers_buf = [0; 2048];
        let mut reader_states = vec![
            // Listen for reader insertions/removals, if supported.
            ReaderState::new(PNP_NOTIFICATION(), pcsc::State::UNAWARE),
        ];

        // spawn a thread to watch for smartcard events
        let handle = thread::spawn({
            let running = running.clone();
            move || {
                loop {
                    if *running.lock().unwrap() == false {
                        break;
                    }

                    // Remove dead readers.
                    fn is_dead(rs: &ReaderState) -> bool {
                        rs.event_state()
                            .intersects(pcsc::State::UNKNOWN | pcsc::State::IGNORE)
                    }
                    for rs in &reader_states {
                        if is_dead(rs) {
                            println!("Removing {:?}", rs.name());
                        }
                    }
                    reader_states.retain(|rs| !is_dead(rs));

                    // Add new readers.
                    let names = ctx
                        .list_readers(&mut readers_buf)
                        .expect("failed to list readers");
                    for name in names {
                        if !reader_states.iter().any(|rs| rs.name() == name) {
                            println!("Adding {:?}", name);
                            reader_states.push(ReaderState::new(name, pcsc::State::UNAWARE));

                            // match ctx.connect(name, ShareMode::Exclusive, Protocols::ANY) {
                            //     Ok(card) => {
                            //         println!("Connected to card in reader {:?}", name);
                            //         // let command = CardCommand::select(OPEN_FIPS_201_AID.to_vec());
                            //         // let mut response_apdu_buf = [0; MAX_APDU_LENGTH];
                            //         // for apdu in command.to_command_apdus() {
                            //         //     println!("Sending APDU: {:?}", apdu);
                            //         //     let response_apdu = card
                            //         //         .transmit(&apdu.data, &mut response_apdu_buf)
                            //         //         .expect("failed to transmit APDU command to card");
                            //         //     println!("APDU response: {:?}", response_apdu);

                            //         //     if response_apdu.starts_with(&SUCCESS) {
                            //         //         println!("Successfully selected OpenFIPS201 applet.");
                            //         //     } else if response_apdu.starts_with(&SUCCESS_MORE_DATA_AVAILABLE) {
                            //         //         println!("Successfully selected OpenFIPS201 applet, and more data is available.");
                            //         //     }
                            //         // }
                            //     }
                            //     Err(pcsc::Error::NoSmartcard) => {
                            //         println!("A smartcard is not present in the reader.");
                            //     }
                            //     Err(err) => {
                            //         eprintln!(
                            //             "Failed to connect to card in reader {:?}: {}",
                            //             name, err
                            //         );
                            //     }
                            // }
                        }
                    }

                    // Update the view of the state to wait on.
                    for rs in &mut reader_states {
                        rs.sync_current_state();
                    }

                    // Wait until the state changes.
                    loop {
                        if *running.lock().unwrap() == false {
                            break;
                        }

                        if !matches!(
                            ctx.get_status_change(Duration::from_millis(100), &mut reader_states),
                            Err(pcsc::Error::Timeout)
                        ) {
                            break;
                        }
                    }

                    // Print current state.
                    println!();
                    *connected.lock().unwrap() = reader_states
                        .iter()
                        .any(|rs| rs.event_state().intersects(pcsc::State::PRESENT));

                    println!("Event Update: Connected: {}", *connected.lock().unwrap());
                    // for rs in &reader_states {
                    //     rs.event_state() & pcsc::State::PRESENT;
                    //     if rs.name() != PNP_NOTIFICATION() {
                    //         println!("{:?} {:?} {:?}", rs.name(), rs.event_state(), rs.atr());
                    //     }
                    // }
                }
            }
        });

        SmartcardWatcherController {
            running,
            handle: Some(handle),
        }
    }
}

fn main() {
    let watcher = SmartcardWatcher::new();
    let controller = watcher.watch();

    for _ in 0..10 {
        println!(
            "Poll Update: Connected: {}",
            *watcher.connected.lock().unwrap()
        );
        thread::sleep(Duration::from_secs(1));
    }

    controller.stop();
}
