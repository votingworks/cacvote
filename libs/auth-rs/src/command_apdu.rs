/// The max length of an APDU
pub const MAX_APDU_LENGTH: usize = 260;

/// The max length of a command APDU's data. The `- 5` accounts for the CLA, INS, P1, P2, and Lc
/// (see CommandApdu below).
pub const MAX_COMMAND_APDU_DATA_LENGTH: usize = MAX_APDU_LENGTH - 5;

/// The max length of a response APDU's data. The `- 2` accounts for the status word (see
/// STATUS_WORD below).
pub const MAX_RESPONSE_APDU_DATA_LENGTH: usize = MAX_APDU_LENGTH - 2;

/// Because APDUs have a max length, commands involving larger amounts of data have to be sent as
/// multiple, chained APDUs. The APDU CLA indicates whether more data has yet to be provided.
///
/// The CLA also indicates whether the APDU is being sent over a GlobalPlatform Secure Channel,
/// typically used for initial card configuration.
#[derive(Debug)]
pub enum CLA {
    Standard = 0x00,
    Chained = 0x10,
    Secure = 0x0c,
    SecureChained = 0x1c,
}

#[derive(Debug)]
pub struct CommandApdu {
    cla: CLA,
    ins: u8,
    p1: u8,
    p2: u8,
    lc: u8,
    data: Vec<u8>,
}

impl CommandApdu {
    pub fn new(cla: CLA, ins: u8, p1: u8, p2: u8, data: Vec<u8>) -> Result<Self, Error> {
        let lc = data.len();

        if lc > MAX_COMMAND_APDU_DATA_LENGTH {
            return Err(Error::DataTooLong { lc });
        }

        Ok(Self {
            cla,
            ins,
            p1,
            p2,
            lc: lc as u8,
            data,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("data is too long: {lc}")]
    DataTooLong { lc: usize },
}
