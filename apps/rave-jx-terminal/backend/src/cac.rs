use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Public};
use openssl::sign::Verifier;
use openssl::x509::X509;
use types_rs::rave::jx::VerificationStatus;

pub(crate) fn verify_cast_vote_record(
    common_access_card_certificate: &[u8],
    cast_vote_record: &[u8],
    cast_vote_record_signature: &[u8],
) -> VerificationStatus {
    let x509 = match X509::from_pem(common_access_card_certificate) {
        Ok(x509) => x509,
        Err(err) => {
            return VerificationStatus::Error(format!(
                "error parsing X509 certificate from PEM format: {err}"
            ));
        }
    };

    let public_key = match x509.public_key() {
        Ok(public_key) => public_key,
        Err(err) => {
            return VerificationStatus::Error(format!(
                "error extracting public key from X509 certificate: {err}"
            ));
        }
    };

    match verify_signature(cast_vote_record, cast_vote_record_signature, &public_key) {
        Ok(true) => {
            // signature is valid, continue
        }
        Ok(false) => {
            return VerificationStatus::Failure;
        }
        Err(err) => {
            return VerificationStatus::Error(format!("error verifying signature: {err}"));
        }
    }

    let Some(CommonNameMetadata {
        common_access_card_id,
        surname,
        given_name,
        middle_name,
    }) = extract_common_name_metadata(&x509) else {
        return VerificationStatus::Error(
            "could not extract and parse CN field from X509 certificate".to_owned()
        );
    };
    let display_name = format!("{surname}, {given_name} {middle_name}")
        .trim()
        .to_owned();

    VerificationStatus::Success {
        common_access_card_id,
        display_name,
    }
}

fn verify_signature(
    message_buffer: &[u8],
    signature_buffer: &[u8],
    public_key: &PKey<Public>,
) -> color_eyre::Result<bool> {
    let mut verifier = Verifier::new(MessageDigest::sha256(), public_key)?;

    // Update the verifier with the message data
    verifier.update(message_buffer)?;

    // Verify the signature
    Ok(verifier.verify(signature_buffer)?)
}

struct CommonNameMetadata {
    common_access_card_id: String,
    surname: String,
    given_name: String,
    middle_name: String,
}

fn extract_common_name_metadata(x509: &X509) -> Option<CommonNameMetadata> {
    // extract the common access card id and name from the certificate
    let common_name_entry = x509
        .subject_name()
        .entries_by_nid(openssl::nid::Nid::COMMONNAME)
        .next()?;

    let common_name_value = common_name_entry
        .data()
        .as_utf8()
        .ok()
        .map(|s| s.to_string())?;

    // extract the common access card id and name from the certificate into a tuple
    // the string is in this format: "SURNAME.FIRSTNAME.MIDDLENAME.ID"
    let common_name_parts: Vec<&str> = common_name_value.split('.').collect();
    if common_name_parts.len() != 4 {
        return None;
    }

    Some(CommonNameMetadata {
        common_access_card_id: common_name_parts[3].trim().to_owned(),
        surname: common_name_parts[0].trim().to_owned(),
        given_name: common_name_parts[1].trim().to_owned(),
        middle_name: common_name_parts[2].trim().to_owned(),
    })
}
