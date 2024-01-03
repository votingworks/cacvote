use crate::{
    command_apdu::{Cla, CommandApdu, MAX_COMMAND_APDU_DATA_LENGTH},
    tlv::{self, Tlv},
};

#[derive(Debug)]
pub struct CardCommand {
    secure_channel: bool,
    ins: u8,
    p1: u8,
    p2: u8,
    data: Vec<u8>,
}

impl CardCommand {
    #[must_use]
    pub fn to_command_apdus(&self) -> Vec<CommandApdu> {
        let mut command_apdus = vec![];

        let mut data = self.data.clone();
        while !data.is_empty() {
            let chunk = data.split_off(usize::min(MAX_COMMAND_APDU_DATA_LENGTH, data.len()));
            let is_last = chunk.is_empty();
            let command_apdu = CommandApdu::new(
                match (self.secure_channel, is_last) {
                    (true, true) => Cla::Secure,
                    (true, false) => Cla::SecureChained,
                    (false, true) => Cla::Standard,
                    (false, false) => Cla::Chained,
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

    #[must_use]
    pub fn to_command_apdu(&self) -> CommandApdu {
        let command_apdus = self.to_command_apdus();
        assert_eq!(command_apdus.len(), 1);
        command_apdus[0].clone()
    }

    /// The SELECT command is a standard command for selecting an applet.
    #[must_use]
    pub fn select(data: impl Into<Vec<u8>>) -> Self {
        Self {
            secure_channel: false,
            ins: 0xa4,
            p1: 0x04,
            p2: 0x00,
            data: data.into(),
        }
    }

    /// The GET RESPONSE command is a standard command for retrieving additional APDU response data.
    #[must_use]
    pub fn get_response(length: u8) -> Self {
        Self {
            secure_channel: false,
            ins: 0xc0,
            p1: 0x00,
            p2: 0x00,
            data: vec![length],
        }
    }

    /// The GET DATA command is a standard command for retrieving data from the card.
    pub fn get_data(object_id: impl Into<Vec<u8>>) -> Result<Self, tlv::ConstructError> {
        Ok(Self {
            secure_channel: false,
            ins: 0xcb,
            p1: 0x3f,
            p2: 0xff,
            data: Tlv::new(0x5c, object_id.into()).try_into()?,
        })
    }
}
