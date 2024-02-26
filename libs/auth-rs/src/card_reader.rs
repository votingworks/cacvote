use std::{
    sync::{mpsc, Arc, Mutex},
    thread::JoinHandle,
    time::Duration,
};

use openssl::x509::X509;
use pcsc::{Card, ReaderState, State, PNP_NOTIFICATION};

use crate::{
    card_details::{self, CardDetails},
    hex_debug::hex_debug,
    tlv::{ConstructError, ParseError, Tlv},
    CardCommand, CommandApdu,
};

/// The OpenFIPS201 applet ID
pub(crate) const OPEN_FIPS_201_AID: [u8; 11] = [
    0xa0, 0x00, 0x00, 0x03, 0x08, 0x00, 0x00, 0x10, 0x00, 0x01, 0x00,
];

pub struct CertObject {
    private_key_id: u8,
}

impl CertObject {
    #[must_use]
    const fn new(private_key_id: u8) -> Self {
        Self { private_key_id }
    }

    /// Data object IDs of the format 0x5f 0xc1 0xXX are a PIV convention.
    #[must_use]
    pub fn object_id(&self) -> Vec<u8> {
        vec![0x5f, 0xc1, self.private_key_id]
    }
}

/// The card's VotingWorks-issued cert
pub const CARD_VX_CERT: CertObject = CertObject::new(0xf0);

/// The card's VxAdmin-issued cert
pub const CARD_VX_ADMIN_CERT: CertObject = CertObject::new(0xf1);

/// The cert authority cert of the VxAdmin that programmed the card
pub const VX_ADMIN_CERT_AUTHORITY_CERT: CertObject = CertObject::new(0xf2);

type StatusWord = [u8; 2];

const SUCCESS: StatusWord = [0x90, 0x00];
const SUCCESS_MORE_DATA_AVAILABLE: StatusWord = [0x61, 0x00];
const VERIFY_FAIL: StatusWord = [0x63, 0x00];
const SECURITY_CONDITION_NOT_SATISFIED: StatusWord = [0x69, 0x82];
const FILE_NOT_FOUND: StatusWord = [0x6a, 0x82];

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

#[derive(Debug)]
struct TransmitResponse {
    data: Vec<u8>,
    more_data: bool,
    more_data_length: u8,
}

impl TryFrom<&[u8]> for TransmitResponse {
    type Error = pcsc::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            [.., sw1, sw2] if *sw1 == SUCCESS[0] && *sw2 == SUCCESS[1] => {
                Ok(Self {
                    data: value[0..value.len() - 2].to_vec(), // trim status word
                    more_data: false,
                    more_data_length: 0,
                })
            }
            [.., sw1, sw2] if *sw1 == SUCCESS_MORE_DATA_AVAILABLE[0] => {
                Ok(Self {
                    data: value[0..value.len() - 2].to_vec(), // trim status word
                    more_data: true,
                    more_data_length: *sw2,
                })
            }
            _ => Err(pcsc::Error::InvalidValue),
        }
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
    #[error("openssl error: {0}")]
    OpenSSL(#[from] openssl::error::ErrorStack),
    #[error("card details error: {0}")]
    CardDetails(#[from] card_details::ParseError),
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

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn read_card_details(&self) -> Result<CardDetails, CardReaderError> {
        let card = self.get_card()?;

        self.select_applet(&card)?;

        let _card_vx_cert = self.retrieve_cert(&card, CARD_VX_CERT.object_id())?;
        let card_vx_admin_cert = self.retrieve_cert(&card, CARD_VX_ADMIN_CERT.object_id())?;
        let _vx_admin_cert_authority_cert =
            self.retrieve_cert(&card, VX_ADMIN_CERT_AUTHORITY_CERT.object_id())?;

        Ok(card_vx_admin_cert.try_into()?)
    }

    fn get_card(&self) -> Result<Card, CardReaderError> {
        tracing::debug!("connecting to card reader: {}", self.name);
        Ok(self.ctx.connect(
            &std::ffi::CString::new(self.name.as_bytes()).unwrap(),
            pcsc::ShareMode::Exclusive,
            pcsc::Protocols::ANY,
        )?)
    }

    #[tracing::instrument(level = "debug", skip(self, card))]
    fn select_applet(&self, card: &Card) -> Result<(), CardReaderError> {
        let command = CardCommand::select(OPEN_FIPS_201_AID);
        self.transmit(card, command)?;
        Ok(())
    }

    fn retrieve_cert(
        &self,
        card: &Card,
        cert_object_id: impl Into<Vec<u8>>,
    ) -> Result<X509, CardReaderError> {
        let cert_object_id = cert_object_id.into();
        tracing::debug!("retrieving cert with object ID: {cert_object_id:02x?}");
        let data = self.get_data(card, cert_object_id)?;
        let cert_tlv = data[0..data.len() - 5].to_vec(); // trim metadata
        let cert_tlv: Tlv = cert_tlv.try_into()?;
        let cert_in_der_format = cert_tlv.value().to_vec();
        Ok(X509::from_der(&cert_in_der_format)?)
    }

    fn get_data(
        &self,
        card: &Card,
        cert_object_id: impl Into<Vec<u8>>,
    ) -> Result<Vec<u8>, CardReaderError> {
        let command = CardCommand::get_data(cert_object_id)?;
        let response = self.transmit(card, command)?;
        let tlv: Tlv = response.try_into()?;
        Ok(tlv.value().to_vec())
    }

    #[tracing::instrument(level = "debug", skip(self, card), fields(card_command = hex_debug(&card_command)))]
    fn transmit(&self, card: &Card, card_command: CardCommand) -> Result<Vec<u8>, pcsc::Error> {
        let mut data: Vec<u8> = vec![];
        let mut more_data = false;
        let mut more_data_length = 0u8;

        for apdu in card_command.to_command_apdus() {
            if apdu.is_chained() {
                self.transmit_helper(card, apdu)?;
            } else {
                let response = self.transmit_helper(card, apdu)?;
                data.extend(response.data);
                more_data = response.more_data;
                more_data_length = response.more_data_length;
            }
        }

        while more_data {
            let response =
                self.transmit_helper(card, CommandApdu::get_response(more_data_length))?;
            data.extend(response.data);
            more_data = response.more_data;
            more_data_length = response.more_data_length;
        }

        Ok(data)
    }

    #[tracing::instrument(level = "debug", skip(self, card), fields(apdu = hex_debug(&apdu)))]
    fn transmit_helper(
        &self,
        card: &Card,
        apdu: CommandApdu,
    ) -> Result<TransmitResponse, pcsc::Error> {
        let mut receive_buffer = [0; 1024];
        let send_buffer = apdu.to_bytes();
        tracing::debug!("sending: {:02x?}", send_buffer);
        let response_apdu = card.transmit(&send_buffer, &mut receive_buffer)?;
        tracing::debug!("received: {:02x?}", response_apdu);

        response_apdu.try_into()
    }
}
