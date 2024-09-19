use std::{
    fmt::Debug,
    io::{sink, Write},
    time::SystemTime,
};

use openssl::x509::X509;
use uuid::Uuid;

use crate::{
    async_card::AsyncCard,
    card_command::{
        GENERAL_AUTHENTICATE_DYNAMIC_TEMPLATE_TAG, GENERAL_AUTHENTICATE_RESPONSE_TAG,
        PUT_DATA_CERT_INFO_TAG, PUT_DATA_CERT_INFO_UNCOMPRESSED, PUT_DATA_CERT_TAG,
        PUT_DATA_DATA_TAG, PUT_DATA_ERROR_DETECTION_CODE_TAG,
    },
    card_details::{extract_field_value, CardDetails, CardDetailsWithAuthInfo, PinInfo},
    card_reader::{CardReaderError, CertObject, OPEN_FIPS_201_AID},
    certs::VX_CUSTOM_CERT_FIELD_COMPONENT,
    hex_debug::hex_debug,
    tlv::Tlv,
    CardCommand, CommandApdu,
};

/// The card's VotingWorks-issued cert
pub const CARD_VX_CERT: CertObject = CertObject::new(0xf0);

/// The card's VxAdmin-issued cert
pub const CARD_VX_ADMIN_CERT: CertObject = CertObject::new(0xf1);

/// The cert authority cert of the VxAdmin that programmed the card
pub const VX_ADMIN_CERT_AUTHORITY_CERT: CertObject = CertObject::new(0xf2);

/// Java Cards always have a PIN. To allow for "PIN-less" cards and "blank"
/// cards, we use a default PIN.
pub const DEFAULT_PIN: &str = "000000";

/// Provides an async interface to JavaCards programmed with the VotingWorks
/// applet. Intended to be used with Tokio.
pub struct VxCard {
    card: AsyncCard,
}

impl Debug for VxCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VxCard").finish()
    }
}

impl VxCard {
    pub const fn new(card: AsyncCard) -> Self {
        Self { card }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn read_card_details(&self) -> Result<CardDetailsWithAuthInfo, CardReaderError> {
        self.select_applet().await?;

        // Verify that the card VotingWorks cert was signed by VotingWorks
        let card_vx_cert = self.retrieve_cert(CARD_VX_CERT.object_id()).await?;
        let vx_cert_authority_cert = openssl::x509::X509::from_pem(include_bytes!(
            "../../../libs/auth/certs/dev/vx-cert-authority-cert.pem"
        ))?;

        let vx_cert_authority_public_key = vx_cert_authority_cert.public_key()?;
        if !card_vx_cert.verify(&vx_cert_authority_public_key)? {
            return Err(CardReaderError::CertificateValidation(
                "card_vx_cert was not signed by vx_cert_authority_cert".to_owned(),
            ));
        }

        // Verify that the card VxAdmin cert was signed by VxAdmin
        let card_vx_admin_cert = self.retrieve_cert(CARD_VX_ADMIN_CERT.object_id()).await?;
        let vx_admin_cert_authority_cert = self
            .retrieve_cert(VX_ADMIN_CERT_AUTHORITY_CERT.object_id())
            .await?;

        let vx_admin_cert_authority_cert_public_key = vx_admin_cert_authority_cert.public_key()?;
        if !card_vx_admin_cert.verify(&vx_admin_cert_authority_cert_public_key)? {
            return Err(CardReaderError::CertificateValidation(
                "card_vx_admin_cert was not signed by vx_admin_cert_authority_cert".to_owned(),
            ));
        }

        // Verify that the VxAdmin cert authority cert on the card is a valid VxAdmin cert, signed by
        // VotingWorks
        let vx_admin_cert_authority_cert = openssl::x509::X509::from_pem(include_bytes!(
            "../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem"
        ))?;

        let vx_admin_cert_authority_cert_component = extract_field_value(
            &vx_admin_cert_authority_cert,
            VX_CUSTOM_CERT_FIELD_COMPONENT,
        )?;

        if !matches!(vx_admin_cert_authority_cert_component, Some(component) if component == "admin")
        {
            return Err(CardReaderError::CertificateValidation(
                "vx_admin_cert_authority_cert was not a valid VxAdmin cert".to_owned(),
            ));
        }

        if !vx_admin_cert_authority_cert.verify(&vx_cert_authority_public_key)? {
            return Err(CardReaderError::CertificateValidation(
                "vx_admin_cert_authority_cert was not signed by vx_cert_authority_cert".to_owned(),
            ));
        }

        // Verify that the card has a private key that corresponds to the public key in the card
        // VotingWorks cert
        let card_vx_cert_public_key = card_vx_cert.public_key()?;
        self.verify_card_private_key(CARD_VX_CERT, &card_vx_cert_public_key, None)
            .await?;

        let card_details: CardDetails = card_vx_admin_cert.clone().try_into()?;

        // If the card doesn't have a PIN:
        // Verify that the card has a private key that corresponds to the public key in the card
        // VxAdmin cert
        //
        // If the card does have a PIN:
        // Perform this verification later in checkPin because operations with this private key are
        // PIN-gated
        let card_does_not_have_pin = matches!(&card_details, CardDetails::PollWorkerCard(card_details) if !card_details.has_pin);

        if card_does_not_have_pin {
            self.verify_card_private_key(
                CARD_VX_ADMIN_CERT,
                &card_vx_admin_cert.public_key()?,
                Some(DEFAULT_PIN),
            )
            .await?;
        }

        let pin_info = if card_does_not_have_pin {
            PinInfo::NoPin
        } else {
            PinInfo::HasPin {
                num_incorrect_pin_attempts: self.get_num_incorrect_pin_attempts().await?,
            }
        };

        Ok(CardDetailsWithAuthInfo::new(
            card_details,
            card_vx_cert,
            card_vx_admin_cert,
            vx_admin_cert_authority_cert,
            pin_info,
        ))
    }

    async fn get_num_incorrect_pin_attempts(&self) -> Result<u8, CardReaderError> {
        let command = CardCommand::get_num_incorrect_pin_attempts();
        match self.transmit(command).await {
            Ok(_) => Ok(0),
            // 63 cx: The counter has reached the value 'cx' (0x00..0x0f)
            Err(CardReaderError::ApduResponse { sw1: 0x63, sw2 }) if sw2 & 0xc0 == 0xc0 => {
                Ok(sw2 & 0x0f)
            }
            Err(e) => Err(e),
        }
    }

    #[tracing::instrument(level = "debug", skip(self, pin))]
    pub async fn check_pin(&self, pin: &str) -> Result<(), CardReaderError> {
        self.select_applet().await?;
        let card_vx_admin_cert = self.retrieve_cert(CARD_VX_ADMIN_CERT.object_id()).await?;

        self.verify_card_private_key(
            CARD_VX_ADMIN_CERT,
            &card_vx_admin_cert.public_key()?,
            Some(pin),
        )
        .await?;

        Ok(())
    }

    async fn check_pin_internal(&self, pin: &str) -> Result<(), CardReaderError> {
        let command = CardCommand::verify_pin(pin);
        self.transmit(command).await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, data, pin))]
    pub async fn sign(
        &self,
        signing_cert: CertObject,
        data: &[u8],
        pin: Option<&str>,
    ) -> Result<(Vec<u8>, X509), CardReaderError> {
        self.select_applet().await?;
        let cert = self.retrieve_cert(signing_cert.object_id()).await?;
        let public_key = cert.public_key()?;
        Ok((
            self.sign_with_keys(signing_cert, &public_key, data, pin)
                .await?,
            cert,
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, public_key, pin))]
    async fn verify_card_private_key(
        &self,
        signing_cert: CertObject,
        public_key: &openssl::pkey::PKey<openssl::pkey::Public>,
        pin: Option<&str>,
    ) -> Result<(), CardReaderError> {
        // have the private key sign a "challenge"
        let challenge_string = format!(
            "VotingWorks/{seconds_since_epoch}/{uuid}",
            seconds_since_epoch = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            uuid = Uuid::new_v4()
        );
        let challenge = challenge_string.as_bytes();
        self.sign_with_keys(signing_cert, public_key, challenge, pin)
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, public_key, data, pin))]
    async fn sign_with_keys(
        &self,
        signing_cert: CertObject,
        public_key: &openssl::pkey::PKey<openssl::pkey::Public>,
        data: &[u8],
        pin: Option<&str>,
    ) -> Result<Vec<u8>, CardReaderError> {
        self.check_pin_internal(pin.unwrap_or(DEFAULT_PIN)).await?;

        let data_hash = openssl::sha::sha256(data);
        let command =
            CardCommand::verify_card_private_key(signing_cert.private_key_id, &data_hash)?;
        let response = self.transmit(command).await?;
        let response_tlv = Tlv::parse(GENERAL_AUTHENTICATE_DYNAMIC_TEMPLATE_TAG, &response)?;
        let general_authenticate_tlv =
            Tlv::parse(GENERAL_AUTHENTICATE_RESPONSE_TAG, response_tlv.value())?;
        let signature = general_authenticate_tlv.value();
        let mut verifier =
            openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), public_key)?;

        verifier.update(data)?;

        if !verifier.verify(signature)? {
            tracing::error!("signature did not verify");
            return Err(CardReaderError::Pcsc(pcsc::Error::InvalidValue));
        }

        Ok(signature.to_vec())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn select_applet(&self) -> Result<(), CardReaderError> {
        self.transmit(CardCommand::select(OPEN_FIPS_201_AID))
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self), fields(card_command = hex_debug(&card_command)))]
    async fn transmit(&self, card_command: CardCommand) -> Result<Vec<u8>, CardReaderError> {
        let mut data: Vec<u8> = vec![];
        let mut more_data = false;
        let mut more_data_length = 0u8;

        for apdu in card_command.to_command_apdus() {
            if apdu.is_chained() {
                self.transmit_helper(apdu, sink()).await?;
            } else {
                match self.transmit_helper(apdu, &mut data).await? {
                    TransmitResponse::Done => {
                        more_data = false;
                    }
                    TransmitResponse::HasMoreData(length) => {
                        more_data = true;
                        more_data_length = length;
                    }
                    TransmitResponse::Other { sw1, sw2 } => {
                        return Err(CardReaderError::ApduResponse { sw1, sw2 });
                    }
                }
            }
        }

        while more_data {
            match self
                .transmit_helper(CommandApdu::get_response(more_data_length), &mut data)
                .await?
            {
                TransmitResponse::Done => {
                    more_data = false;
                }
                TransmitResponse::HasMoreData(length) => {
                    more_data_length = length;
                }
                TransmitResponse::Other { sw1, sw2 } => {
                    return Err(CardReaderError::ApduResponse { sw1, sw2 });
                }
            }
        }

        Ok(data)
    }

    #[tracing::instrument(level = "debug", skip(self, buffer), fields(apdu = hex_debug(&apdu)))]
    async fn transmit_helper(
        &self,
        apdu: CommandApdu,
        mut buffer: impl Write,
    ) -> Result<TransmitResponse, CardReaderError> {
        tracing::debug!("sending (APDU): {:02x?}", apdu);
        let send_buffer = apdu.to_bytes();
        tracing::debug!("sending (bytes): {:02x?}", send_buffer);
        let response_apdu = self.card.transmit(send_buffer).await?;
        tracing::debug!("received: {:02x?}", response_apdu);

        let (data, result) = TransmitResponse::parse(&response_apdu)?;
        buffer.write_all(data)?;
        Ok(result)
    }

    #[tracing::instrument(level = "debug", skip(self, cert_object_id))]
    async fn retrieve_cert(
        &self,
        cert_object_id: impl Into<Vec<u8>>,
    ) -> Result<X509, CardReaderError> {
        let cert_object_id = cert_object_id.into();
        tracing::debug!("retrieving cert with object ID: {cert_object_id:02x?}");
        let data = self.get_data(cert_object_id).await?;
        let (remainder, cert_tlv) = Tlv::parse_partial(PUT_DATA_CERT_TAG, &data)?;
        let (remainder, cert_info_tlv) = Tlv::parse_partial(PUT_DATA_CERT_INFO_TAG, &remainder)?;

        if !matches!(cert_info_tlv.value(), [PUT_DATA_CERT_INFO_UNCOMPRESSED]) {
            tracing::error!("unexpected cert_info_tlv: {cert_info_tlv:02x?}");
            return Err(CardReaderError::Pcsc(pcsc::Error::InvalidValue));
        }

        let error_detection_code_tlv = Tlv::parse(PUT_DATA_ERROR_DETECTION_CODE_TAG, &remainder)?;

        if !error_detection_code_tlv.value().is_empty() {
            tracing::error!("unexpected error_detection_code_tlv: {error_detection_code_tlv:02x?}");
            return Err(CardReaderError::Pcsc(pcsc::Error::InvalidValue));
        }

        let cert_in_der_format = cert_tlv.value();
        Ok(X509::from_der(cert_in_der_format)?)
    }

    async fn get_data(
        &self,
        cert_object_id: impl Into<Vec<u8>>,
    ) -> Result<Vec<u8>, CardReaderError> {
        let command = CardCommand::get_data(cert_object_id)?;
        let response = self.transmit(command).await?;
        let tlv = Tlv::parse(PUT_DATA_DATA_TAG, &response)?;
        Ok(tlv.value().to_vec())
    }

    pub async fn disconnect(self) -> Result<(), CardReaderError> {
        self.card.disconnect().await?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum TransmitResponse {
    Done,
    HasMoreData(u8),
    Other { sw1: u8, sw2: u8 },
}

impl TransmitResponse {
    pub fn parse(data: &[u8]) -> Result<(&[u8], Self), CardReaderError> {
        match data {
            [data @ .., 0x61, length] => Ok((data, Self::HasMoreData(*length))),
            [data @ .., 0x90, 0x00] => Ok((data, Self::Done)),
            [data @ .., sw1, sw2] => Ok((
                data,
                Self::Other {
                    sw1: *sw1,
                    sw2: *sw2,
                },
            )),
            _ => Err(CardReaderError::Pcsc(pcsc::Error::InvalidValue)),
        }
    }
}
