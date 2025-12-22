//! Transitive Attestation decoding for Blocky attestations.
//!
//! This module provides `no_std`-compatible parsing of transitive attestation
//! blobs, which are ABI-encoded as `bytes[]` with exactly 2 elements:
//! - Element 0: data (the claims payload)
//! - Element 1: signature (65 bytes: r, s, v - we extract first 64 bytes)

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

#[cfg(feature = "std")]
use std::{boxed::Box, vec::Vec};

use ethabi::{decode, ParamType, Token};

/// Errors that can occur during transitive attestation decoding.
#[derive(Debug, Clone, PartialEq)]
pub enum TransitiveAttestationError {
    /// Failed to ABI-decode the input bytes.
    AbiDecodingFailed,
    /// The decoded array does not have exactly 2 elements.
    InvalidArrayLength,
    /// Failed to extract the data element from the array.
    MissingDataElement,
    /// Failed to extract the signature element from the array.
    MissingSignatureElement,
    /// The signature is shorter than 64 bytes (r + s).
    SignatureTooShort,
}

/// A decoded transitive attestation containing the data and signature components.
#[derive(Debug, Clone)]
pub struct DecodedTransitiveAttestation {
    /// The claims data payload.
    pub data: Vec<u8>,
    /// The signature r,s components (64 bytes total).
    pub signature_rs: Vec<u8>,
}

impl DecodedTransitiveAttestation {
    /// Returns the data payload.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the 64-byte r,s signature components.
    pub fn signature_rs(&self) -> &[u8] {
        &self.signature_rs
    }
}

/// Decodes a transitive attestation blob into its data and signature components.
///
/// The input is expected to be ABI-encoded as `bytes[]` with exactly 2 elements:
/// - `[0]`: The data/claims payload
/// - `[1]`: The signature (at least 65 bytes: r[32] + s[32] + v[1])
///
/// This function extracts only the first 64 bytes of the signature (r and s),
/// discarding the recovery ID (v) byte.
///
/// # Arguments
///
/// * `ta` - The raw transitive attestation bytes
///
/// # Returns
///
/// A `DecodedTransitiveAttestation` containing the data and 64-byte signature,
/// or an error if decoding fails.
///
/// # Example
///
/// ```ignore
/// let ta_bytes = /* base64-decoded transitive_attestation field */;
/// let decoded = decode_transitive_attestation(&ta_bytes)?;
/// let data = decoded.data();
/// let sig_rs = decoded.signature_rs();
/// ```
pub fn decode_transitive_attestation(
    ta: &[u8],
) -> Result<DecodedTransitiveAttestation, TransitiveAttestationError> {
    // ABI decode as bytes[]
    let decoded = decode(&[ParamType::Array(Box::new(ParamType::Bytes))], ta)
        .map_err(|_| TransitiveAttestationError::AbiDecodingFailed)?
        .pop()
        .and_then(|t| t.into_array())
        .ok_or(TransitiveAttestationError::AbiDecodingFailed)?;

    // Require exactly 2 elements
    if decoded.len() != 2 {
        return Err(TransitiveAttestationError::InvalidArrayLength);
    }

    // Extract data (element 0)
    let data = extract_bytes(&decoded[0])
        .ok_or(TransitiveAttestationError::MissingDataElement)?;

    // Extract signature (element 1)
    let sig_bytes = extract_bytes(&decoded[1])
        .ok_or(TransitiveAttestationError::MissingSignatureElement)?;

    // Signature must be at least 64 bytes (r + s)
    if sig_bytes.len() < 64 {
        return Err(TransitiveAttestationError::SignatureTooShort);
    }

    // Keep only first 64 bytes (r and s, discard v)
    let signature_rs = sig_bytes[..64].to_vec();

    Ok(DecodedTransitiveAttestation { data, signature_rs })
}

/// Helper to extract bytes from an ethabi Token.
fn extract_bytes(token: &Token) -> Option<Vec<u8>> {
    token.clone().into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_output_for_tests;

    #[test]
    fn test_decode_transitive_attestation() {
        let output = block_output_for_tests();
        let ta = output.ta();

        // The TA struct already decoded the raw bytes, so we need the raw TA bytes
        // from the BlockyOutput. Let's decode from the original transitive_attestation field.
        use base64::prelude::*;
        let ta_bytes = BASE64_STANDARD
            .decode(&output.transitive_attested_function_call.transitive_attestation)
            .expect("Failed to decode base64");

        let decoded = decode_transitive_attestation(&ta_bytes)
            .expect("Failed to decode transitive attestation");

        // Verify we got data
        assert!(!decoded.data.is_empty(), "Data should not be empty");

        // Verify signature is exactly 64 bytes
        assert_eq!(decoded.signature_rs.len(), 64, "Signature should be 64 bytes");

        // The decoded data should match what TA.data() returns
        assert_eq!(decoded.data(), ta.data(), "Data should match TA.data()");

        // The signature_rs should match TA.signature_bytes()
        assert_eq!(decoded.signature_rs(), ta.signature_bytes().as_slice(), "Signature should match");
    }

    #[test]
    fn test_decode_invalid_data() {
        // Empty input
        let result = decode_transitive_attestation(&[]);
        assert!(matches!(result, Err(TransitiveAttestationError::AbiDecodingFailed)));

        // Random garbage
        let result = decode_transitive_attestation(&[1, 2, 3, 4]);
        assert!(matches!(result, Err(TransitiveAttestationError::AbiDecodingFailed)));
    }
}
