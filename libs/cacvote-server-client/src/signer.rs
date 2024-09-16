use std::{fmt::Debug, io::Write, path::PathBuf, process::Command};

use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private},
};

use crate::result::{Error, Result};

pub trait Signer {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>>;
}

pub type AnySigner = Box<dyn Signer + Send + Sync>;

/// A `Signer` that uses a private key to sign payloads.
#[derive(Debug)]
#[must_use]
pub struct PrivateKeySigner {
    private_key: PKey<Private>,
}

impl PrivateKeySigner {
    pub fn new(private_key: PKey<Private>) -> Self {
        Self { private_key }
    }
}

impl Signer for PrivateKeySigner {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let mut signer = openssl::sign::Signer::new(MessageDigest::sha256(), &self.private_key)
            .map_err(|e| Error::Signature(e.to_string()))?;
        signer
            .update(payload)
            .map_err(|e| Error::Signature(e.to_string()))?;
        signer
            .sign_to_vec()
            .map_err(|e| Error::Signature(e.to_string()))
    }
}

/// A signer that uses the TPM to sign payloads.
pub struct TpmSigner {
    handle: u32,
}

impl TpmSigner {
    pub const fn new(handle: u32) -> Self {
        Self { handle }
    }
}

impl Debug for TpmSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TpmSigner")
            .field("handle", &format!("{:x}", self.handle))
            .finish()
    }
}

impl Signer for TpmSigner {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        // NOTE: you must have the tpm2 OpenSSL provider installed for this to
        // work, and the user running the program must have access to the TPM.
        // Add the user to the `tss` group to allow access to the TPM.
        let mut command = Command::new("openssl")
            .arg("dgst")
            .arg("-sha256")
            .arg("-sign")
            .arg(format!("handle:0x{:x}", self.handle))
            .arg("-provider")
            .arg("tpm2")
            .arg("-provider")
            .arg("default")
            .spawn()
            .map_err(|e| Error::Signature(e.to_string()))?;

        let Some(mut stdin) = command.stdin.take() else {
            return Err(Error::Signature(
                "openssl stdin is not available".to_string(),
            ));
        };
        stdin
            .write_all(payload)
            .map_err(|e| Error::Signature(e.to_string()))?;

        let output = command
            .wait_with_output()
            .map_err(|e| Error::Signature(e.to_string()))?;

        if !output.status.success() {
            return Err(Error::Signature(format!(
                "openssl failed with exit code: {}",
                output.status
            )));
        }

        Ok(output.stdout)
    }
}

/// A description of a signer.
#[derive(Debug, Clone)]
pub enum Description {
    /// The path of a private key file.
    File(PathBuf),

    /// The handle of a TPM key.
    Tpm(u32),
}

impl std::str::FromStr for Description {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix("tpm:") {
            Some(handle) => match handle.strip_prefix("0x") {
                Some(handle) => {
                    let handle = u32::from_str_radix(handle, 16).map_err(|e| e.to_string())?;
                    Ok(Self::Tpm(handle))
                }
                None => {
                    let handle = handle.parse::<u32>().map_err(|e| e.to_string())?;
                    Ok(Self::Tpm(handle))
                }
            },
            None => Ok(Self::File(PathBuf::from(s))),
        }
    }
}

impl TryFrom<Description> for AnySigner {
    type Error = color_eyre::Report;

    fn try_from(value: Description) -> Result<Self, Self::Error> {
        match value {
            Description::File(path) => {
                let pem = std::fs::read(path)?;
                Ok(Box::new(PrivateKeySigner::new(PKey::private_key_from_pem(
                    &pem,
                )?)))
            }
            Description::Tpm(handle) => Ok(Box::new(TpmSigner::new(handle))),
        }
    }
}
