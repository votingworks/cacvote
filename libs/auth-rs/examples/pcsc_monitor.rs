use std::{sync::mpsc, thread, time::Duration};

// Example of how to monitor card & card reader state changes.
use pcsc::*;

fn pcsc_thread(stop_rx: mpsc::Receiver<()>) {
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    let mut readers_buf = [0; 2048];
    let mut reader_states = vec![
        // Listen for reader insertions/removals, if supported.
        ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE),
    ];
    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        // Remove dead readers.
        fn is_dead(rs: &ReaderState) -> bool {
            rs.event_state().intersects(State::UNKNOWN | State::IGNORE)
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
                reader_states.push(ReaderState::new(name, State::UNAWARE));
            }
        }

        // Update the view of the state to wait on.
        for rs in &mut reader_states {
            rs.sync_current_state();
        }

        'status_change: loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }

            println!(
                "about to call get_status_change: {:?}",
                reader_states
                    .iter()
                    .map(|rs| (rs.name(), rs.event_state(), rs.atr()))
                    .collect::<Vec<_>>()
            );

            // Wait until the state changes.
            match ctx.get_status_change(Duration::from_secs(1), &mut reader_states) {
                Ok(()) => {
                    break 'status_change;
                }
                Err(Error::Timeout) => {}
                Err(err) => {
                    println!("error: {:?}", err);
                    break 'status_change;
                }
            }
        }

        println!(
            "after status change reader states: {:?}",
            reader_states
                .iter()
                .map(|rs| (rs.name(), rs.event_state(), rs.atr()))
                .collect::<Vec<_>>()
        );

        // Print current state.
        println!();
        for rs in &reader_states {
            if rs.name() != PNP_NOTIFICATION() {
                println!("{:?} {:?} {:?}", rs.name(), rs.event_state(), rs.atr());
            }
        }
    }
}

fn main() {
    let (stop_tx, stop_rx) = mpsc::channel();
    let handle = thread::spawn(move || pcsc_thread(stop_rx));

    for _ in 0..10 {
        thread::sleep(Duration::from_secs(1));
    }

    stop_tx.send(()).unwrap();
    handle.join().unwrap();
}
