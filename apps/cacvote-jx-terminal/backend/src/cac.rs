use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Public};
use openssl::sign::Verifier;
use openssl::x509::X509;
use types_rs::cacvote::jx::VerificationStatus;

const DOD_TEST_CAC_CERTIFICATE_BYTES: &[u8] = include_bytes!("../certs/DODJITCEMAILCA_63.cer");

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CertificateAuthority {
    DodTest,
}

pub(crate) fn verify_cast_vote_record(
    common_access_card_certificate: &[u8],
    cast_vote_record: &[u8],
    cast_vote_record_signature: &[u8],
    ca: CertificateAuthority,
) -> VerificationStatus {
    let common_access_card_certificate = match X509::from_pem(common_access_card_certificate) {
        Ok(x509) => x509,
        Err(err) => {
            return VerificationStatus::Error(format!(
                "error parsing X509 certificate from PEM format: {err}"
            ));
        }
    };

    match verify_common_access_card_certificate(&common_access_card_certificate, ca) {
        Ok(true) => {
            // certificate is valid, continue
        }
        Ok(false) => {
            return VerificationStatus::Failure;
        }
        Err(err) => {
            return VerificationStatus::Error(format!(
                "error verifying CAC certificate against {ca:?} CA: {err}"
            ));
        }
    }

    let public_key = match common_access_card_certificate.public_key() {
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
    }) = extract_common_name_metadata(&common_access_card_certificate)
    else {
        return VerificationStatus::Error(
            "could not extract and parse CN field from X509 certificate".to_owned(),
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

/// Verify that the certificate is signed by the given certificate authority.
/// This function only supports the DOD Test CA.
fn verify_common_access_card_certificate(
    common_access_card_certificate: &X509,
    ca: CertificateAuthority,
) -> color_eyre::Result<bool> {
    match ca {
        CertificateAuthority::DodTest => {
            let signing_cert = match X509::from_der(DOD_TEST_CAC_CERTIFICATE_BYTES) {
                Ok(signing_cert) => signing_cert,
                Err(err) => {
                    return Err(color_eyre::eyre::eyre!(
                        "error parsing {ca:?} certificate from DER format: {err}"
                    ));
                }
            };

            let public_key = match signing_cert.public_key() {
                Ok(public_key) => public_key,
                Err(err) => {
                    return Err(color_eyre::eyre::eyre!(
                        "error extracting public key from {ca:?} certificate: {err}"
                    ));
                }
            };

            // directly verifies the certificate against the DOD Test CA because
            // we're only verifying against the one certificate for now. when we
            // need to verify the whole chain, we'll need to use the appropriate
            // openssl APIs to verify the chain or perform this check in a loop
            Ok(common_access_card_certificate.verify(&public_key)?)
        }
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

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod test {
    use super::*;
    use openssl::x509::{X509Builder, X509NameBuilder};
    use pretty_assertions::assert_eq;
    use proptest::{prop_assert_eq, proptest};

    #[test]
    fn extract_metadata_from_sample_cac_cert() {
        let cert_bytes = include_bytes!("../tests/fixtures/robert_aikins_sample_cert.pem");
        let x509 = X509::from_pem(cert_bytes).unwrap();
        assert_eq!(
            extract_common_name_metadata(&x509),
            Some(CommonNameMetadata {
                common_access_card_id: "1404922102".to_owned(),
                surname: "AIKINS".to_owned(),
                given_name: "ROBERT".to_owned(),
                middle_name: "EDDIE".to_owned(),
            }),
        );
    }

    proptest! {
        #[test]
        /// Generate a random X509 certificate and verify that the common name
        /// metadata can be extracted from it. The names are limited to 17
        /// characters because the maximum length of the CN field is 64 bytes,
        /// the Common Access Card ID is 10 digits, and the separators are 3
        /// characters. 64 - 10 - 3 = 51 available bytes → 51 / 3 = 17
        /// characters each.
        fn test_generated_x509(
            common_access_card_id in "\\d{10}",
            given_name in "[A-Z]{1,17}",
            surname in "[A-Z]{1,17}",
            middle_name in "[A-Z]{0,17}",
        ) {
            let mut subject_name = X509NameBuilder::new().unwrap();
            subject_name.append_entry_by_nid(
                openssl::nid::Nid::COMMONNAME,
                format!("{surname}.{given_name}.{middle_name}.{common_access_card_id}").as_str(),
            ).unwrap();
            let mut x509_builder = X509Builder::new().unwrap();
            x509_builder.set_subject_name(&subject_name.build()).unwrap();
            let x509 = x509_builder.build();

            prop_assert_eq!(
                extract_common_name_metadata(&x509),
                Some(CommonNameMetadata {
                    common_access_card_id,
                    surname,
                    given_name,
                    middle_name,
                })
            );
        }

        #[test]
        /// Generate a random X509 certificate and verify that the common name
        /// metadata cannot be extracted from it because the CN field is
        /// invalid.
        fn test_invalid_common_names(
            common_name in "[^\\.]{1,64}",
        ) {
            let mut subject_name = X509NameBuilder::new().unwrap();
            subject_name.append_entry_by_nid(
                openssl::nid::Nid::COMMONNAME,
                common_name.as_str(),
            ).unwrap();
            let mut x509_builder = X509Builder::new().unwrap();
            x509_builder.set_subject_name(&subject_name.build()).unwrap();
            let x509 = x509_builder.build();

            prop_assert_eq!(
                extract_common_name_metadata(&x509),
                None
            );
        }
    }
}
