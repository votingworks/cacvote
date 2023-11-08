use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

use openssl::{error, x509::X509};
use pcsc::{Card, ReaderState, State, PNP_NOTIFICATION};

use crate::{
    card_command,
    command_apdu::CLA,
    tlv::{ConstructError, ParseError, Tlv},
    CardCommand, CommandApdu,
};

/// The OpenFIPS201 applet ID
const OPEN_FIPS_201_AID: [u8; 11] = [
    0xa0, 0x00, 0x00, 0x03, 0x08, 0x00, 0x00, 0x10, 0x00, 0x01, 0x00,
];

/// Data object IDs of the format 0x5f 0xc1 0xXX are a PIV convention.
const fn piv_data_object_id(unique_byte: u8) -> [u8; 3] {
    [0x5f, 0xc1, unique_byte]
}

pub struct CertInfo {
    object_id: [u8; 3],
    private_key_id: u8,
}

/// The card's VotingWorks-issued cert
pub const CARD_VX_CERT: CertInfo = CertInfo {
    object_id: piv_data_object_id(0xf0),
    private_key_id: 0xf0,
};

/// The card's VxAdmin-issued cert
pub const CARD_VX_ADMIN_CERT: CertInfo = CertInfo {
    object_id: piv_data_object_id(0xf1),
    private_key_id: 0xf1,
};

/// The cert authority cert of the VxAdmin that programmed the card
pub const VX_ADMIN_CERT_AUTHORITY_CERT: CertInfo = CertInfo {
    object_id: piv_data_object_id(0xf2),
    private_key_id: 0xf2,
};

type StatusWord = [u8; 2];

const SUCCESS: StatusWord = [0x90, 0x00];
const SUCCESS_MORE_DATA_AVAILABLE: StatusWord = [0x61, 0x00];
const VERIFY_FAIL: StatusWord = [0x63, 0x00];
const SECURITY_CONDITION_NOT_SATISFIED: StatusWord = [0x69, 0x82];
const FILE_NOT_FOUND: StatusWord = [0x6a, 0x82];

#[derive(Debug)]
pub enum Event {
    ReaderAdded { name: String },
    ReaderRemoved { name: String },
    CardInserted { reader: String },
    CardRemoved { reader: String },
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

#[derive(Debug)]
struct TransmitResponse {
    data: Vec<u8>,
    more_data: bool,
    more_data_length: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum CardReaderError {
    #[error("construct error: {0}")]
    ConstructError(#[from] ConstructError),
    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),
    #[error("pcsc error: {0}")]
    PcscError(#[from] pcsc::Error),
    #[error("openssl error: {0}")]
    OpensslError(#[from] openssl::error::ErrorStack),
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
                        tx.send(Ok(Event::ReaderRemoved {
                            name: rs.name().to_str().unwrap().to_owned(),
                        }))
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
                            name: name.to_str().unwrap().to_owned(),
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
                        tx.send(Ok(Event::CardInserted {
                            reader: rs.name().to_str().unwrap().to_owned(),
                        }))
                        .unwrap();
                    }

                    if rs
                        .event_state()
                        .contains(pcsc::State::CHANGED | pcsc::State::EMPTY)
                    {
                        // card removed
                        tx.send(Ok(Event::CardRemoved {
                            reader: rs.name().to_str().unwrap().to_owned(),
                        }))
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

    pub fn read_card_details(&self, reader: String) -> Result<(), CardReaderError> {
        let card = self.ctx.connect(
            &std::ffi::CString::new(reader).unwrap(),
            pcsc::ShareMode::Exclusive,
            pcsc::Protocols::ANY,
        )?;

        let now = Instant::now();
        self.select_applet(&card)?;
        println!("select_applet took: {:?}", now.elapsed());
        let card_vx_cert = self.retrieve_cert(&card, CARD_VX_CERT.object_id.clone())?;

        println!("card_vx_cert: {:#?}", card_vx_cert);

        let card_vx_admin_cert = self.retrieve_cert(&card, CARD_VX_ADMIN_CERT.object_id.clone())?;

        println!("card_vx_admin_cert: {:#?}", card_vx_admin_cert);

        let vx_admin_cert_authority_cert =
            self.retrieve_cert(&card, VX_ADMIN_CERT_AUTHORITY_CERT.object_id.clone())?;

        println!(
            "vx_admin_cert_authority_cert: {:#?}",
            vx_admin_cert_authority_cert
        );

        Ok(())
    }

    fn select_applet(&self, card: &Card) -> Result<(), CardReaderError> {
        let command = CardCommand::select(OPEN_FIPS_201_AID);
        let now = Instant::now();
        self.transmit(card, command)?;
        println!("select_applet.transmit took: {:?}", now.elapsed());
        Ok(())
    }

    fn retrieve_cert(
        &self,
        card: &Card,
        cert_object_id: impl Into<Vec<u8>>,
    ) -> Result<X509, CardReaderError> {
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

    fn transmit(&self, card: &Card, card_command: CardCommand) -> Result<Vec<u8>, pcsc::Error> {
        let mut data: Vec<u8> = vec![];
        let mut more_data = false;
        let mut more_data_length = 0u8;

        for apdu in card_command.to_command_apdus() {
            if apdu.is_chained() {
                self.transmit_helper(&card, apdu)?;
            } else {
                let response = self.transmit_helper(&card, apdu)?;
                data.extend(response.data);
                more_data = response.more_data;
                more_data_length = response.more_data_length;
            }
        }

        while more_data {
            let get_response_command = CardCommand::get_response(more_data_length);
            let apdu = get_response_command.to_command_apdu();
            let response = self.transmit_helper(&card, apdu)?;
            data.extend(response.data);
            more_data = response.more_data;
            more_data_length = response.more_data_length;
        }

        Ok(data)
    }

    fn transmit_helper(
        &self,
        card: &Card,
        apdu: CommandApdu,
    ) -> Result<TransmitResponse, pcsc::Error> {
        let mut receive_buffer = [0; 1024];
        let mut send_buffer = apdu.to_bytes();

        println!("send_buffer: {:02x?}", send_buffer);
        let now = Instant::now();
        let response_apdu = card.transmit(&mut send_buffer, &mut receive_buffer)?;
        println!("card.transmit took: {:?}", now.elapsed());
        println!("response_apdu: {:02x?}", response_apdu);

        if response_apdu.len() < 2 {
            return Err(pcsc::Error::InvalidValue);
        }

        let data = response_apdu.split_at(response_apdu.len() - 2).0;
        let sw1 = response_apdu[response_apdu.len() - 2];
        let sw2 = response_apdu[response_apdu.len() - 1];

        if sw1 == SUCCESS[0] && sw2 == SUCCESS[1] {
            Ok(TransmitResponse {
                data: data.to_vec(),
                more_data: false,
                more_data_length: 0,
            })
        } else if sw1 == SUCCESS_MORE_DATA_AVAILABLE[0] {
            Ok(TransmitResponse {
                data: data.to_vec(),
                more_data: true,
                more_data_length: sw2,
            })
        } else {
            Err(pcsc::Error::UnknownError)
        }
    }
}
