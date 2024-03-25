use std::{
    sync::{mpsc, Arc, Mutex},
    thread::JoinHandle,
    time::Duration,
};

use pcsc::{ReaderState, State, PNP_NOTIFICATION};

use crate::{
    card_details,
    tlv::{ConstructError, ParseError},
    vx_card::VxCard,
};

/// The OpenFIPS201 applet ID
pub(crate) const OPEN_FIPS_201_AID: [u8; 11] = [
    0xa0, 0x00, 0x00, 0x03, 0x08, 0x00, 0x00, 0x10, 0x00, 0x01, 0x00,
];

pub struct CertObject {
    pub private_key_id: u8,
}

impl CertObject {
    #[must_use]
    pub const fn new(private_key_id: u8) -> Self {
        Self { private_key_id }
    }

    /// Data object IDs of the format 0x5f 0xc1 0xXX are a PIV convention.
    #[must_use]
    pub fn object_id(&self) -> Vec<u8> {
        vec![0x5f, 0xc1, self.private_key_id]
    }
}

pub type SharedCardReaders = Arc<Mutex<Vec<String>>>;

#[derive(Debug)]
pub enum Event {
    ReaderAdded { reader_name: String },
    ReaderRemoved { reader_name: String },
    CardInserted { reader_name: String },
    CardRemoved { reader_name: String },
}

pub struct Watcher {
    handle: Option<JoinHandle<()>>,
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
        let (stop_watch_tx, stop_watch_rx) = mpsc::channel();
        let (tx, rx) = mpsc::channel();

        let mut readers_buf = [0; 2048];
        let mut reader_states = vec![
            // Listen for reader insertions/removals, if supported.
            ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE),
        ];

        let card_readers: SharedCardReaders = Arc::new(Mutex::new(vec![]));

        let handle = std::thread::spawn({
            let card_readers = card_readers.clone();
            move || {
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
                            tx.send(Ok(Event::ReaderAdded {
                                reader_name: name.to_str().unwrap().to_owned(),
                            }))
                            .unwrap();
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

                        // NOTE: the timeout duration here is important. If it's too short, added
                        // card readers will not be detected. If it's too long, the watcher will
                        // not stop in a timely manner.
                        match ctx.get_status_change(Duration::from_secs(1), &mut reader_states) {
                            Ok(()) => {
                                break 'status_change;
                            }
                            Err(pcsc::Error::Timeout) => {
                                // no change, keep waiting
                            }
                            Err(err) => {
                                tx.send(Err(err)).unwrap();
                                break 'status_change;
                            }
                        }
                    }

                    for rs in &reader_states {
                        if rs.name() == PNP_NOTIFICATION() || is_dead(rs) {
                            continue;
                        }

                        if rs
                            .event_state()
                            .contains(pcsc::State::CHANGED | pcsc::State::PRESENT)
                        {
                            // found a card
                            let name = rs.name().to_str().unwrap().to_owned();
                            let reader_already_in_list = card_readers
                                .lock()
                                .unwrap()
                                .iter()
                                .any(|reader| **reader == name);

                            if !reader_already_in_list {
                                let mut card_readers = card_readers.lock().unwrap();
                                card_readers.push(name.clone());
                            }
                            tx.send(Ok(Event::CardInserted { reader_name: name }))
                                .unwrap();
                        }

                        if rs
                            .event_state()
                            .contains(pcsc::State::CHANGED | pcsc::State::EMPTY)
                        {
                            // card removed
                            let name = rs.name().to_str().unwrap().to_owned();
                            let mut card_readers = card_readers.lock().unwrap();
                            card_readers.retain(|r| *r != name);
                            tx.send(Ok(Event::CardRemoved { reader_name: name }))
                                .unwrap();
                        }
                    }
                }
            }
        });

        Watcher {
            handle: Some(handle),
            stop_watch: stop_watch_tx,
            receiver: rx,
            card_readers,
        }
    }

    pub fn stop(&mut self) {
        self.stop_watch.send(()).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }

    pub fn events(&self) -> &mpsc::Receiver<Result<Event, pcsc::Error>> {
        &self.receiver
    }

    pub fn readers_with_cards(&self) -> SharedCardReaders {
        self.card_readers.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CardReaderError {
    #[error("construct error: {0}")]
    Construct(#[from] ConstructError),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("pc/sc error: {0}")]
    Pcsc(#[from] pcsc::Error),
    #[error("APDU response error: [{sw1}, {sw2}]")]
    ApduResponse { sw1: u8, sw2: u8 },
    #[error("openssl error: {0}")]
    OpenSSL(#[from] openssl::error::ErrorStack),
    #[error("card details error: {0}")]
    CardDetails(#[from] card_details::ParseError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("certificate validation error: {0}")]
    CertificateValidation(String),
}

impl CardReaderError {
    pub fn is_incorrect_pin_error(&self) -> bool {
        matches!(self, Self::ApduResponse { sw1: 0x63, .. })
    }
}

#[derive(Clone)]
pub struct CardReader {
    ctx: pcsc::Context,
    name: String,
}

impl std::fmt::Debug for CardReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CardReader")
            .field("name", &self.name)
            .finish()
    }
}

impl CardReader {
    pub fn new(ctx: pcsc::Context, reader_name: String) -> Self {
        Self {
            ctx,
            name: reader_name,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_card(&self) -> Result<VxCard, CardReaderError> {
        tracing::debug!("connecting to card reader: {}", self.name);
        Ok(VxCard::new(self.ctx.connect(
            &std::ffi::CString::new(self.name.as_bytes()).unwrap(),
            pcsc::ShareMode::Exclusive,
            pcsc::Protocols::ANY,
        )?))
    }
}
