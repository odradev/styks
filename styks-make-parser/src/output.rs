use std::{fs, path::Path};

use crate::Data;
use base64::{Engine, prelude::BASE64_STANDARD};
use casper_types::{AsymmetricType, bytesrepr::ToBytes};
use k256::sha2::{Digest, Sha256};
use serde::Deserialize;

pub type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Deserialize, Debug)]
pub struct SignedPriceData {
    pub request: String,
    pub response_body: String,
    pub public_key: String,
    pub nonce: String,
    pub signature: String,
    pub timestamp: String,
}

impl SignedPriceData {
    pub fn try_from_file<P: AsRef<Path>>(path: P) -> Result<Self, DynError> {
        let text = fs::read_to_string(path)?;
        let parsed: Data<SignedPriceData> = serde_json_wasm::from_str(&text)?;
        Ok(parsed.data)
    }

    pub fn public_key(&self) -> Result<casper_types::PublicKey, VerificationError> {
        let public_key_bytes = BASE64_STANDARD
            .decode(&self.public_key)
            .map_err(|_| VerificationError::InvalidPublicKey)?;
        let raw_key = &public_key_bytes[public_key_bytes.len() - 32..];
        casper_types::PublicKey::ed25519_from_bytes(&raw_key)
            .map_err(|_| VerificationError::InvalidPublicKey)
    }

    pub fn signature_bytes(&self) -> Result<Vec<u8>, VerificationError> {
        let bytes = BASE64_STANDARD
            .decode(&self.signature)
            .map_err(|_| VerificationError::InvalidSignature)?;
        let sig = casper_types::crypto::Signature::ed25519_from_bytes(&bytes)
            .map_err(|_| VerificationError::InvalidSignature)?;
        sig.to_bytes()
            .map_err(|_| VerificationError::InvalidSignature)
    }

    pub fn payload_bytes(&self) -> Vec<u8> {
        let payload = crate::Payload::from(self);
        serde_json::to_vec(&payload).expect("Failed to serialize payload.")
    }

    pub fn verify_signature(&self) -> Result<(), VerificationError> {
        let signature_bytes = BASE64_STANDARD
            .decode(&self.signature)
            .map_err(|_| VerificationError::InvalidSignature)?;
        let signature = casper_types::crypto::Signature::ed25519_from_bytes(&signature_bytes)
            .map_err(|_| VerificationError::InvalidSignature)?;
        let pk = self.public_key()?;
        let payload = crate::Payload::from(self);

        let payload_bytes =
            serde_json::to_vec(&payload).map_err(|_| VerificationError::HashingError)?;
        let hashed_payload = Sha256::digest(payload_bytes);
        casper_types::crypto::verify(hashed_payload, &signature, &pk)
            .map_err(|_| VerificationError::BadSignature)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerificationError {
    InvalidPublicKey,
    InvalidSignature,
    HashingError,
    BadSignature,
}
