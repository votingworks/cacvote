use crate::command_apdu::{CommandApdu, CLA, MAX_COMMAND_APDU_DATA_LENGTH};

#[derive(Debug)]
pub struct CardCommand {
    secure_channel: bool,
    ins: u8,
    p1: u8,
    p2: u8,
    data: Vec<u8>,
}

impl CardCommand {
    pub fn to_command_apdus(&self) -> Vec<CommandApdu> {
        let mut command_apdus = vec![];

        let mut data = self.data.clone();
        while !data.is_empty() {
            let chunk = data.split_off(usize::min(MAX_COMMAND_APDU_DATA_LENGTH, data.len()));
            let is_last = data.is_empty();
            let command_apdu = CommandApdu::new(
                match (self.secure_channel, is_last) {
                    (true, true) => CLA::Secure,
                    (true, false) => CLA::SecureChained,
                    (false, true) => CLA::Standard,
                    (false, false) => CLA::Chained,
                },
                self.ins,
                self.p1,
                self.p2,
                data,
            )
            .expect("data is less than MAX_COMMAND_APDU_DATA_LENGTH");
            command_apdus.push(command_apdu);
            data = chunk;
        }

        command_apdus
    }

    /// The SELECT command is a standard command for selecting an applet.
    pub const fn select(data: Vec<u8>) -> Self {
        Self {
            secure_channel: false,
            ins: 0xa4,
            p1: 0x04,
            p2: 0x00,
            data,
        }
    }

    /// The GET RESPONSE command is a standard command for retrieving additional APDU response data.
    pub fn get_response(length: u8) -> Self {
        Self {
            secure_channel: false,
            ins: 0xc0,
            p1: 0x00,
            p2: 0x00,
            data: vec![length],
        }
    }
}
