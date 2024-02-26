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
    pub fn new(ins: u8, p1: u8, p2: u8, data: Vec<u8>) -> Self {
        Self {
            secure_channel: false,
            ins,
            p1,
            p2,
            data,
        }
    }

    /// The SELECT command is a standard command for selecting an applet.
    #[must_use]
    pub fn select(data: impl Into<Vec<u8>>) -> Self {
        Self::new(0xa4, 0x04, 0x00, data.into())
    }

    /// The GET DATA command is a standard command for retrieving data from the card.
    pub fn get_data(object_id: impl Into<Vec<u8>>) -> Result<Self, tlv::ConstructError> {
        Ok(Self::new(
            0xcb,
            0x3f,
            0xff,
            Tlv::new(0x5c, object_id.into()).try_into()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::card_reader::OPEN_FIPS_201_AID;

    use super::*;

    #[test]
    fn test_select() {
        let command = CardCommand::select(OPEN_FIPS_201_AID);
        let apdus = command.to_command_apdus();
        assert_eq!(
            apdus,
            vec![
                CommandApdu::new(Cla::Standard, 0xa4, 0x04, 0x00, OPEN_FIPS_201_AID.to_vec())
                    .unwrap(),
            ]
        );
    }
}
