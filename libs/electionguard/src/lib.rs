#![deny(clippy::all)]

use std::path::PathBuf;

use electionguard_rs::{ballot, config, manifest};
use napi::bindgen_prelude::*;
use types_rs::{cdf::cvr::Cvr, election::Election};

#[macro_use]
extern crate napi_derive;

// NOTE: The functions below use `serde_json::Value` as input/output types
// because the `napi` macro does not support bulk-renaming to use `snake_case`
// like `serde_json` does. Also, some struct fields are not defined in a
// crate that we own (such as `time::Date`), so we cannot easily implement
// `ToNapiValue` and `FromNapiValue` for them.

/// Convert a VX election to an EG manifest. The return value is the EG manifest
/// as a POJO.
#[napi(
    ts_args_type = "vxElection: import('@votingworks/types').Election",
    ts_return_type = "import('./types').Manifest"
)]
pub fn convert_vx_election_to_eg_manifest(
    vx_election: serde_json::Value,
) -> Result<serde_json::Value> {
    let vx_election: Election = serde_json::from_value(vx_election).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to parse VX election: {e}"),
        )
    })?;

    let eg_manifest: manifest::Manifest = vx_election.into();
    Ok(serde_json::to_value(eg_manifest).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to serialize EG manifest: {e}"),
        )
    })?)
}

#[napi(object)]
pub struct ElectionConfig {
    pub public_metadata_blob: Buffer,
    pub private_metadata_blob: Buffer,
}

#[napi(ts_args_type = "classpath: string, egManifest: import('./types').Manifest")]
pub fn generate_election_config(
    classpath: String,
    eg_manifest: serde_json::Value,
) -> Result<ElectionConfig> {
    let eg_manifest: manifest::Manifest = serde_json::from_value(eg_manifest).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to parse EG manifest: {e}"),
        )
    })?;

    let config::ElectionConfig {
        public_metadata_blob,
        private_metadata_blob,
    } = config::generate_election_config(&PathBuf::from(classpath), eg_manifest).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to generate election config: {e}"),
        )
    })?;

    Ok(ElectionConfig {
        public_metadata_blob: public_metadata_blob.into(),
        private_metadata_blob: private_metadata_blob.into(),
    })
}

/// Convert a VX CVR to an EG plaintext ballot. The `vx_election` argument
/// must be the election that corresponds to the `vx_cvr` argument. The
/// `eg_manifest` argument must be the manifest that corresponds to the
/// `vx_election` argument. The return value is the EG plaintext ballot as a
/// POJO.
#[napi(
    ts_args_type = "vxElection: import('@votingworks/types').Election, serialNumber: number, egManifest: import('./types').Manifest, vxCvr: import('@votingworks/types').CVR.CVR",
    ts_return_type = "import('./types').PlaintextBallot"
)]
pub fn convert_vx_cvr_to_eg_plaintext_ballot(
    vx_election: serde_json::Value,
    serial_number: serde_json::Number,
    eg_manifest: serde_json::Value,
    vx_cvr: serde_json::Value,
) -> Result<serde_json::Value> {
    let eg_manifest: manifest::Manifest = serde_json::from_value(eg_manifest).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to parse EG manifest: {e}"),
        )
    })?;

    let vx_election: Election = serde_json::from_value(vx_election).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to parse VX election: {e}"),
        )
    })?;

    let converted_manifest: manifest::Manifest = vx_election.clone().into();

    if eg_manifest != converted_manifest {
        return Err(Error::new(
            Status::GenericFailure,
            "VX election and EG manifest do not match",
        ));
    }

    let vx_cvr: Cvr = serde_json::from_value(vx_cvr).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to parse VX CVR: {e}"),
        )
    })?;

    let serial_number = match serial_number.to_string().parse::<f64>() {
        Ok(serial_number) => serial_number.round() as u64,
        Err(e) => {
            return Err(Error::new(
                Status::GenericFailure,
                format!("Failed to parse serial number: {e}"),
            ));
        }
    };

    let plaintext_ballot = ballot::convert_vx_cvr_to_eg_plaintext_ballot(
        vx_cvr,
        serial_number,
        eg_manifest,
        vx_election,
    )
    .map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to convert VX CVR to EG plaintext ballot: {e}"),
        )
    })?;

    serde_json::to_value(plaintext_ballot).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to serialize EG plaintext ballot: {e}"),
        )
    })
}

/// Encrypt an EG plaintext ballot using the ElectionGuard Java implementation.
/// The `classpath` argument should be the path to the ElectionGuard Java JAR
/// file.  The `public_metadata_blob` argument should be the public metadata
/// blob from the `ElectionConfig` struct. The return value is the encrypted
/// ballot as a POJO.
#[napi(
    ts_args_type = "classpath: string, publicMetadataBlob: Uint8Array, egPlaintextBallot: import('./types').PlaintextBallot, deviceName: string",
    ts_return_type = "import('./types').EncryptedBallot"
)]
pub fn encrypt_eg_plaintext_ballot(
    classpath: String,
    public_metadata_blob: &[u8],
    mut eg_plaintext_ballot: serde_json::Value,
    device_name: String,
) -> Result<serde_json::Value> {
    let classpath = PathBuf::from(classpath);

    if let serde_json::Value::Object(object) = &mut eg_plaintext_ballot {
        if let Some(serde_json::Value::Number(sn)) = object.get("sn") {
            if sn.is_f64() {
                let sn = sn.as_f64().unwrap().round() as u64;
                let sn = serde_json::Value::Number(sn.into());
                assert!(!sn.is_f64(), "Failed to convert serial number to integer");
                object.insert("sn".to_string(), sn);
            }
        }
    }

    let eg_plaintext_ballot: ballot::PlaintextBallot = serde_json::from_value(eg_plaintext_ballot)
        .map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to parse EG plaintext ballot: {e}"),
            )
        })?;

    let encrypted_ballot_bytes = ballot::encrypt(
        &classpath,
        public_metadata_blob,
        &eg_plaintext_ballot,
        &device_name,
    )
    .map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to encrypt EG plaintext ballot: {e}"),
        )
    })?;

    serde_json::from_slice(&encrypted_ballot_bytes).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to deserialize encrypted ballot: {e}"),
        )
    })
}

/// Extract the ElectionGuard manifest from the public metadata blob.
#[napi(
    ts_args_type = "publicMetadataBlob: Uint8Array",
    ts_return_type = "import('./types').Manifest"
)]
pub fn extract_manifest_from_public_metadata_blob(
    public_metadata_blob: &[u8],
) -> Result<serde_json::Value> {
    let manifest = config::extract_manifest_from_public_metadata_blob(public_metadata_blob)
        .map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to extract manifest from public metadata blob: {e}"),
            )
        })?;

    serde_json::to_value(manifest).map_err(|e| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to serialize manifest: {e}"),
        )
    })
}
