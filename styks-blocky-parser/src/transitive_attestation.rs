//! Transitive Attestation (TA) decoding module.
//!
//! This module provides no_std-compatible parsing of Blocky's transitive attestation
//! data structure, which is ABI-encoded as `bytes[]` containing two elements:
//! - Element 0: The signed data (claims)
//! - Element 1: The signature (65 bytes: r + s + recovery_id)

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

use ethabi::{decode, ParamType};

/// Errors that can occur during transitive attestation decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TAError {
    /// Failed to ABI decode the transitive attestation bytes.
    DecodeFailed,
    /// The decoded array does not have exactly 2 elements.
    InvalidLength,
    /// Failed to extract bytes from the decoded token.
    BytesConversionError,
    /// Signature is too short (expected at least 64 bytes).
    SignatureTooShort,
}

/// Decodes a transitive attestation blob into its data and signature components.
///
/// # Arguments
/// * `ta_bytes` - The raw bytes of the transitive attestation (ABI-encoded `bytes[]`)
///
/// # Returns
/// A tuple of (data, signature) where:
/// - `data` is the signed claims data
/// - `signature` is the 64-byte ECDSA signature (r + s, without recovery ID)
///
/// # Example
/// ```ignore
/// let (data, signature) = decode_transitive_attestation(&ta_bytes)?;
/// // Use data with BlockyClaims::decode_fn_call_claims
/// // Use signature with verify::verify_signature
/// ```
pub fn decode_transitive_attestation(ta_bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), TAError> {
    // ABI decode as bytes[]
    let decoded = decode(&[ParamType::Array(Box::new(ParamType::Bytes))], ta_bytes)
        .map_err(|_| TAError::DecodeFailed)?;

    // Extract the array
    let bytes_array = decoded
        .into_iter()
        .next()
        .ok_or(TAError::DecodeFailed)?
        .into_array()
        .ok_or(TAError::DecodeFailed)?;

    // Expect exactly 2 elements: data and signature
    if bytes_array.len() != 2 {
        return Err(TAError::InvalidLength);
    }

    // Extract data (element 0)
    let data = bytes_array[0]
        .clone()
        .into_bytes()
        .ok_or(TAError::BytesConversionError)?;

    // Extract signature (element 1)
    let sig_bytes = bytes_array[1]
        .clone()
        .into_bytes()
        .ok_or(TAError::BytesConversionError)?;

    // Signature must be at least 64 bytes (r + s)
    // It may be 65 bytes with recovery ID, but we only need 64
    if sig_bytes.len() < 64 {
        return Err(TAError::SignatureTooShort);
    }

    // Return only the first 64 bytes (r + s), stripping recovery ID if present
    let signature = sig_bytes[..64].to_vec();

    Ok((data, signature))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_empty_fails() {
        let result = decode_transitive_attestation(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_abi_fails() {
        let result = decode_transitive_attestation(&[0x00, 0x01, 0x02]);
        assert!(result.is_err());
    }
}

#[cfg(all(test, feature = "std"))]
mod std_tests {
    use super::*;
    use crate::{block_output_for_tests, blocky_claims::BlockyClaims};
    use base64::{prelude::BASE64_STANDARD, Engine};

    #[test]
    fn test_decode_from_blocky_output() {
        // Load test data
        let blocky_output = block_output_for_tests();

        // Get the raw TA bytes (base64 decode)
        let ta_base64 = &blocky_output.transitive_attested_function_call.transitive_attestation;
        let ta_bytes = BASE64_STANDARD.decode(ta_base64).expect("Failed to decode base64");

        // Decode using our function
        let (data, signature) = decode_transitive_attestation(&ta_bytes)
            .expect("Failed to decode TA");

        // Verify data can be parsed as claims
        let claims = BlockyClaims::decode_fn_call_claims(&data)
            .expect("Failed to decode claims from TA data");

        // Verify the claims match expected values
        assert_eq!(claims.function(), "priceFunc");
        assert!(claims.hash_of_code().starts_with("baadaf"));

        // Verify signature is 64 bytes
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_roundtrip_with_existing_ta() {
        // Load test data
        let blocky_output = block_output_for_tests();

        // Get the raw TA bytes
        let ta_base64 = &blocky_output.transitive_attested_function_call.transitive_attestation;
        let ta_bytes = BASE64_STANDARD.decode(ta_base64).expect("Failed to decode base64");

        // Decode using our new function
        let (data, signature) = decode_transitive_attestation(&ta_bytes)
            .expect("Failed to decode TA");

        // Compare with existing TA parsing
        let existing_ta = blocky_output.ta();

        // Data should match
        assert_eq!(data, existing_ta.data());

        // Signature should match (first 64 bytes)
        assert_eq!(signature, existing_ta.signature_bytes());
    }
}
