//! Enclave Attestation parsing module.
//!
//! This module provides no_std-compatible parsing of Blocky's enclave attestation
//! claims structure to extract the measurement (PCRs) and public key.
//!
//! The contract uses "PCR extraction" approach - it trusts Blocky's verification
//! and just extracts the measurement and public key from the claims.

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

/// Errors that can occur during enclave attestation parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnclaveAttestationError {
    /// Failed to parse the JSON claims.
    JsonParseFailed,
    /// Invalid base64 encoding for public key.
    InvalidBase64,
    /// Missing required field.
    MissingField,
}

/// Parsed enclave attestation data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnclaveAttestation {
    /// The attestation platform (e.g., "nitro").
    pub platform: String,
    /// The measurement code (PCR values, e.g., "pcr0.pcr1.pcr2").
    pub measurement_code: String,
    /// The public key in SEC1 format (raw bytes, decoded from base64).
    pub public_key: Vec<u8>,
    /// The curve type (e.g., "p256k1").
    pub curve_type: String,
}

/// JSON structure for enclave attestation claims.
/// This matches the `claims` field in Blocky's `enclave_attested_application_public_key`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveAttestationClaims {
    pub enclave_measurement: EnclaveMeasurement,
    pub public_key: PublicKeyInfo,
}

/// Enclave measurement (platform and PCR code).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveMeasurement {
    pub platform: String,
    pub code: String,
}

/// Public key information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyInfo {
    pub curve_type: String,
    pub data: String, // Base64-encoded SEC1 public key
}

/// Parses enclave attestation claims from JSON bytes.
///
/// # Arguments
/// * `claims_json` - JSON bytes representing `EnclaveAttestationClaims`
/// * `public_key_bytes` - Pre-decoded public key bytes (SEC1 format)
///
/// This function expects the public key to be passed separately as raw bytes
/// because base64 decoding in no_std is complex. The CLI should decode the
/// public key before calling the contract.
///
/// # Returns
/// Parsed `EnclaveAttestation` containing platform, measurement code, and public key.
pub fn parse_enclave_attestation_with_pubkey(
    claims_json: &[u8],
    public_key_bytes: Vec<u8>,
) -> Result<EnclaveAttestation, EnclaveAttestationError> {
    // Parse the JSON claims
    let claims: EnclaveAttestationClaims =
        serde_json_wasm::from_slice(claims_json).map_err(|_| EnclaveAttestationError::JsonParseFailed)?;

    Ok(EnclaveAttestation {
        platform: claims.enclave_measurement.platform,
        measurement_code: claims.enclave_measurement.code,
        public_key: public_key_bytes,
        curve_type: claims.public_key.curve_type,
    })
}

/// Parses enclave attestation claims from JSON bytes (std feature only).
///
/// This version handles base64 decoding of the public key internally.
#[cfg(feature = "std")]
pub fn parse_enclave_attestation(claims_json: &[u8]) -> Result<EnclaveAttestation, EnclaveAttestationError> {
    use base64::{prelude::BASE64_STANDARD, Engine};

    // Parse the JSON claims
    let claims: EnclaveAttestationClaims =
        serde_json_wasm::from_slice(claims_json).map_err(|_| EnclaveAttestationError::JsonParseFailed)?;

    // Decode the base64 public key
    let public_key = BASE64_STANDARD
        .decode(&claims.public_key.data)
        .map_err(|_| EnclaveAttestationError::InvalidBase64)?;

    Ok(EnclaveAttestation {
        platform: claims.enclave_measurement.platform,
        measurement_code: claims.enclave_measurement.code,
        public_key,
        curve_type: claims.public_key.curve_type,
    })
}

/// Validates that an attestation matches an allowed measurement.
///
/// # Arguments
/// * `attestation` - The parsed attestation
/// * `allowed_platform` - Expected platform (e.g., "nitro")
/// * `allowed_code` - Expected measurement code (PCR values)
///
/// # Returns
/// `true` if the attestation matches the allowed measurement.
pub fn validate_measurement(
    attestation: &EnclaveAttestation,
    allowed_platform: &str,
    allowed_code: &str,
) -> bool {
    attestation.platform == allowed_platform && attestation.measurement_code == allowed_code
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CLAIMS_JSON: &[u8] = br#"{
        "enclave_measurement": {
            "platform": "nitro",
            "code": "pcr0.pcr1.pcr2"
        },
        "public_key": {
            "curve_type": "p256k1",
            "data": "dGVzdF9wdWJrZXk="
        }
    }"#;

    #[test]
    fn test_parse_with_pubkey() {
        let pubkey = vec![1, 2, 3, 4];
        let result = parse_enclave_attestation_with_pubkey(TEST_CLAIMS_JSON, pubkey.clone());

        assert!(result.is_ok());
        let attestation = result.unwrap();
        assert_eq!(attestation.platform, "nitro");
        assert_eq!(attestation.measurement_code, "pcr0.pcr1.pcr2");
        assert_eq!(attestation.public_key, pubkey);
        assert_eq!(attestation.curve_type, "p256k1");
    }

    #[test]
    fn test_validate_measurement_match() {
        let attestation = EnclaveAttestation {
            platform: "nitro".into(),
            measurement_code: "abc.def.ghi".into(),
            public_key: vec![],
            curve_type: "p256k1".into(),
        };

        assert!(validate_measurement(&attestation, "nitro", "abc.def.ghi"));
        assert!(!validate_measurement(&attestation, "other", "abc.def.ghi"));
        assert!(!validate_measurement(&attestation, "nitro", "xxx.yyy.zzz"));
    }

    #[test]
    fn test_invalid_json_fails() {
        let result = parse_enclave_attestation_with_pubkey(b"not json", vec![]);
        assert_eq!(result, Err(EnclaveAttestationError::JsonParseFailed));
    }
}

#[cfg(all(test, feature = "std"))]
mod std_tests {
    use super::*;
    use crate::block_output_for_tests;

    #[test]
    fn test_parse_from_blocky_output() {
        let blocky_output = block_output_for_tests();

        // Serialize the claims to JSON
        let claims = &blocky_output.enclave_attested_application_public_key.claims;
        let claims_json = serde_json_wasm::to_string(claims).expect("Failed to serialize claims");

        // Parse using our function
        let attestation = parse_enclave_attestation(claims_json.as_bytes())
            .expect("Failed to parse attestation");

        // Verify platform
        assert_eq!(attestation.platform, "nitro");

        // Verify measurement code format (should be 3 dot-separated PCR hashes)
        let parts: Vec<&str> = attestation.measurement_code.split('.').collect();
        assert_eq!(parts.len(), 3);

        // Each PCR should be 96 hex chars (48 bytes)
        for part in &parts {
            assert_eq!(part.len(), 96);
        }

        // Verify public key is 65 bytes (uncompressed SEC1 for secp256k1)
        assert_eq!(attestation.public_key.len(), 65);
        assert_eq!(attestation.public_key[0], 0x04); // Uncompressed point prefix
    }
}
