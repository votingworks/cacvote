use crate::{
    command_apdu::{Cla, CommandApdu, MAX_COMMAND_APDU_DATA_LENGTH},
    concat_bytes, tlv,
};

pub const SELECT_INS: u8 = 0xa4;
pub const SELECT_P1: u8 = 0x04;
pub const SELECT_P2: u8 = 0x00;
pub const GET_DATA_INS: u8 = 0xcb;
pub const GET_DATA_P1: u8 = 0x3f;
pub const GET_DATA_P2: u8 = 0xff;
pub const GET_DATA_TAG: u8 = 0x5c;
#[allow(dead_code)]
pub const PUT_DATA_INS: u8 = 0xdb;
#[allow(dead_code)]
pub const PUT_DATA_P1: u8 = 0x3f;
#[allow(dead_code)]
pub const PUT_DATA_P2: u8 = 0xff;
pub const PUT_DATA_CERT_TAG: u8 = 0x70;
pub const PUT_DATA_DATA_TAG: u8 = 0x53;
pub const PUT_DATA_CERT_INFO_TAG: u8 = 0x71;
pub const PUT_DATA_CERT_INFO_UNCOMPRESSED: u8 = 0x00;
pub const PUT_DATA_ERROR_DETECTION_CODE_TAG: u8 = 0xfe;
pub const VERIFY_PIN_INS: u8 = 0x20;
pub const VERIFY_PIN_P1: u8 = 0x00;
pub const VERIFY_PIN_P2: u8 = 0x80;
pub const GENERAL_AUTHENTICATE_INS: u8 = 0x87;
pub const GENERAL_AUTHENTICATE_DYNAMIC_TEMPLATE_TAG: u8 = 0x7c;
pub const GENERAL_AUTHENTICATE_CHALLENGE_TAG: u8 = 0x81;
pub const GENERAL_AUTHENTICATE_RESPONSE_TAG: u8 = 0x82;
pub const CRYPTOGRAPHIC_ALGORITHM_ECC256: u8 = 0x11;

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
        Self::new(SELECT_INS, SELECT_P1, SELECT_P2, data.into())
    }

    /// The GET DATA command is a standard command for retrieving data from the card.
    pub fn get_data(object_id: impl Into<Vec<u8>>) -> Result<Self, tlv::ConstructError> {
        Ok(Self::new(
            GET_DATA_INS,
            GET_DATA_P1,
            GET_DATA_P2,
            tlv!(GET_DATA_TAG, object_id),
        ))
    }

    #[tracing::instrument]
    pub fn verify_card_private_key(
        private_key_id: u8,
        challenge_hash: &[u8],
    ) -> Result<Self, tlv::ConstructError> {
        Ok(Self::new(
            GENERAL_AUTHENTICATE_INS,
            CRYPTOGRAPHIC_ALGORITHM_ECC256,
            private_key_id,
            tlv!(
                GENERAL_AUTHENTICATE_DYNAMIC_TEMPLATE_TAG,
                concat_bytes![
                    tlv!(GENERAL_AUTHENTICATE_CHALLENGE_TAG, challenge_hash),
                    tlv!(GENERAL_AUTHENTICATE_RESPONSE_TAG, Vec::new()),
                ]
            ),
        ))
    }

    #[tracing::instrument]
    pub fn verify_pin(pin: &str) -> Self {
        let mut data = [0xffu8; 8];
        let pin_bytes = pin.as_bytes();
        assert!(pin_bytes.len() <= data.len(), "PIN too long");
        data[..pin_bytes.len()].copy_from_slice(pin_bytes);
        tracing::debug!("PIN bytes: {:?}", pin_bytes);
        Self::new(VERIFY_PIN_INS, VERIFY_PIN_P1, VERIFY_PIN_P2, data.to_vec())
    }

    #[tracing::instrument]
    pub fn get_num_incorrect_pin_attempts() -> Self {
        Self::new(VERIFY_PIN_INS, VERIFY_PIN_P1, VERIFY_PIN_P2, Vec::new())
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
