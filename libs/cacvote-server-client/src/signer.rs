use std::{fmt::Debug, io::Write, process::Command};

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
