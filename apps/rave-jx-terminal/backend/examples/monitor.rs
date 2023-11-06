// Example of how to monitor card & card reader state changes.
use pcsc::*;

/// The OpenFIPS201 applet ID
const OPEN_FIPS_201_AID: [u8; 11] = [
    0xa0, 0x00, 0x00, 0x03, 0x08, 0x00, 0x00, 0x10, 0x00, 0x01, 0x00,
];

/**
 * APDU status words are analogous to HTTP status codes. Every response APDU ends with one, each
 * consisting of two bytes, commonly referred to as SW1 and SW2.
 *
 * - 0x90 0x00 is equivalent to an HTTP 200.
 * - 0x61 0xXX is also equivalent to an HTTP 200 but indicates that XX more bytes of response data
 *   have yet to be retrieved via a GET RESPONSE command. Like command APDUs, response APDUs have a
 *   max length.
 *
 * See https://www.eftlab.com/knowledge-base/complete-list-of-apdu-responses for a list of all
 * known status words.
 */
type StatusWord = [u8; 2];

const SUCCESS: StatusWord = [0x90, 0x00];
const SUCCESS_MORE_DATA_AVAILABLE: StatusWord = [0x61, 0x00];
const VERIFY_FAIL: StatusWord = [0x63, 0x00];
const SECURITY_CONDITION_NOT_SATISFIED: StatusWord = [0x69, 0x82];
const FILE_NOT_FOUND: StatusWord = [0x6a, 0x82];

fn main() {
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    let mut readers_buf = [0; 2048];
    let mut reader_states = vec![
        // Listen for reader insertions/removals, if supported.
        ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE),
    ];
    loop {
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

                match ctx.connect(name, ShareMode::Exclusive, Protocols::ANY) {
                    Ok(card) => {
                        println!("Connected to card in reader {:?}", name);
                        let command = CardCommand::select(OPEN_FIPS_201_AID.to_vec());
                        let mut response_apdu_buf = [0; MAX_APDU_LENGTH];
                        for apdu in command.to_command_apdus() {
                            println!("Sending APDU: {:?}", apdu);
                            let response_apdu = card
                                .transmit(&apdu.data, &mut response_apdu_buf)
                                .expect("failed to transmit APDU command to card");
                            println!("APDU response: {:?}", response_apdu);

                            if response_apdu.starts_with(&SUCCESS) {
                                println!("Successfully selected OpenFIPS201 applet.");
                            } else if response_apdu.starts_with(&SUCCESS_MORE_DATA_AVAILABLE) {
                                println!("Successfully selected OpenFIPS201 applet, and more data is available.");
                            }
                        }
                    }
                    Err(Error::NoSmartcard) => {
                        println!("A smartcard is not present in the reader.");
                    }
                    Err(err) => {
                        eprintln!("Failed to connect to card in reader {:?}: {}", name, err);
                    }
                }
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
        println!();
        for rs in &reader_states {
            if rs.name() != PNP_NOTIFICATION() {
                println!("{:?} {:?} {:?}", rs.name(), rs.event_state(), rs.atr());
            }
        }
    }
}
