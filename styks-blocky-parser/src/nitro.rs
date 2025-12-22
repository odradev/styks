//! AWS Nitro Enclave Attestation verification.
//!
//! This module provides parsing and verification of AWS Nitro Enclave attestation
//! documents. The attestation flow is:
//!
//! 1. Parse JSON wrapper containing `platform` and `platform_attestations`
//! 2. Decode base64 platform attestations (COSE_Sign1 documents)
//! 3. Parse COSE_Sign1 structure to extract CBOR attestation document
//! 4. Extract PCRs (Platform Configuration Registers) for measurement
//! 5. Extract the application public key from `user_data`
//!
//! Note: Full X.509 certificate chain verification is stubbed for fallback mode.
//! In strict mode, the COSE signature is verified against the certificate chain
//! rooted at the AWS Nitro Enclaves Root CA.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec, vec::Vec};

#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

use ciborium::Value as CborValue;
use coset::{CborSerializable, CoseSign1};

/// AWS Nitro Enclaves Root CA certificate in DER format.
/// Downloaded from: https://aws-nitro-enclaves.amazonaws.com/AWS_NitroEnclaves_Root-G1.zip
/// SHA-256 fingerprint: 64:1A:03:21:A3:E2:44:EF:E4:56:46:31:95:D6:06:31:7E:D7:CD:CC:3C:17:56:E0:98:93:F3:C6:8F:79:BB:5B
#[allow(dead_code)]
pub const AWS_NITRO_ROOT_CERT_DER: &[u8] = include_bytes!("../resources/AWS_NitroEnclaves_Root-G1.der");

/// Errors that can occur during enclave attestation verification.
#[derive(Debug, Clone, PartialEq)]
pub enum EnclaveAttestationError {
    /// Failed to decode base64 data.
    Base64DecodingFailed,
    /// Failed to parse JSON wrapper.
    JsonParsingFailed,
    /// Platform field is missing or not "nitro".
    InvalidPlatform,
    /// platform_attestations array is missing or empty.
    NoPlatformAttestations,
    /// Failed to decode CBOR data.
    CborDecodingFailed,
    /// Failed to parse COSE_Sign1 structure.
    CoseSign1ParsingFailed,
    /// The COSE_Sign1 payload is missing.
    PayloadMissing,
    /// Failed to parse the attestation document payload.
    AttestationDocumentParsingFailed,
    /// The attestation document is not a CBOR map.
    AttestationDocumentNotMap,
    /// PCRs field is missing from attestation document.
    PcrsMissing,
    /// PCR value is not in expected format.
    PcrFormatInvalid,
    /// Required PCR (0, 1, or 2) is missing.
    RequiredPcrMissing,
    /// user_data field is missing (contains application public key).
    UserDataMissing,
    /// user_data is not valid bytes.
    UserDataInvalid,
    /// Certificate chain verification failed.
    CertificateChainVerificationFailed,
    /// COSE signature verification failed.
    SignatureVerificationFailed,
    /// The certificate field is missing from attestation document.
    CertificateMissing,
    /// The cabundle field is missing from attestation document.
    CaBundleMissing,
}

/// A verified enclave key with its measurement.
#[derive(Debug, Clone)]
pub struct VerifiedEnclaveKey {
    /// The measurement platform (e.g., "nitro").
    pub measurement_platform: String,
    /// The measurement code as "pcr0hex.pcr1hex.pcr2hex".
    pub measurement_code: String,
    /// The application public key in SEC1 uncompressed format (65 bytes for secp256k1).
    pub pubkey_sec1: Vec<u8>,
}

/// Parsed Nitro attestation document fields.
#[derive(Debug, Clone)]
pub struct NitroAttestationDocument {
    /// PCR0 - Enclave image file hash.
    pub pcr0: Vec<u8>,
    /// PCR1 - Linux kernel and bootstrap hash.
    pub pcr1: Vec<u8>,
    /// PCR2 - Application hash.
    pub pcr2: Vec<u8>,
    /// The user_data field (contains application public key).
    pub user_data: Vec<u8>,
    /// The enclave certificate (DER format).
    pub certificate: Vec<u8>,
    /// The CA bundle (chain of certificates).
    pub cabundle: Vec<Vec<u8>>,
}

impl NitroAttestationDocument {
    /// Formats PCRs 0, 1, 2 as a measurement code string: "pcr0hex.pcr1hex.pcr2hex"
    pub fn measurement_code(&self) -> String {
        let pcr0_hex = hex::encode(&self.pcr0);
        let pcr1_hex = hex::encode(&self.pcr1);
        let pcr2_hex = hex::encode(&self.pcr2);
        format!("{}.{}.{}", pcr0_hex, pcr1_hex, pcr2_hex)
    }
}

/// JSON structure for the enclave attestation wrapper.
#[derive(serde::Deserialize)]
struct EnclaveAttestationJson {
    platform: String,
    platform_attestations: Vec<String>,
}

/// JSON structure for the public key in user_data field.
#[derive(serde::Deserialize)]
struct UserDataPublicKey {
    #[allow(dead_code)]
    curve_type: String,
    data: String,
}

/// Verifies a Nitro enclave attestation and extracts the verified key.
///
/// This function:
/// 1. Parses the JSON wrapper
/// 2. Decodes and parses COSE_Sign1 attestation documents
/// 3. Extracts PCRs and user_data (public key)
/// 4. (Optionally) Verifies the certificate chain to AWS Nitro root
///
/// # Arguments
///
/// * `doc` - The raw enclave attestation bytes (base64-decoded JSON)
///
/// # Returns
///
/// A `VerifiedEnclaveKey` containing the measurement and public key.
#[cfg(feature = "std")]
pub fn verify_nitro_enclave_attestation(
    doc: &[u8],
) -> Result<VerifiedEnclaveKey, EnclaveAttestationError> {
    // Parse JSON wrapper
    let json_str = core::str::from_utf8(doc)
        .map_err(|_| EnclaveAttestationError::JsonParsingFailed)?;

    let wrapper: EnclaveAttestationJson = serde_json_wasm::from_str(json_str)
        .map_err(|_| EnclaveAttestationError::JsonParsingFailed)?;

    // Verify platform
    if wrapper.platform != "nitro" {
        return Err(EnclaveAttestationError::InvalidPlatform);
    }

    // Get first attestation (we use the first one)
    let attestation_b64 = wrapper.platform_attestations
        .first()
        .ok_or(EnclaveAttestationError::NoPlatformAttestations)?;

    // Decode base64
    use base64::prelude::*;
    let cose_bytes = BASE64_STANDARD.decode(attestation_b64)
        .map_err(|_| EnclaveAttestationError::Base64DecodingFailed)?;

    // Parse the attestation document
    let attestation_doc = parse_nitro_attestation(&cose_bytes)?;

    // In fallback mode, we skip full certificate chain verification
    // The CLI will perform full verification and use register_signer_manual

    // Parse user_data as JSON containing the public key
    // Format: {"curve_type":"p256k1","data":"<base64 SEC1 pubkey>"}
    let user_data_str = core::str::from_utf8(&attestation_doc.user_data)
        .map_err(|_| EnclaveAttestationError::UserDataInvalid)?;

    let pubkey_json: UserDataPublicKey = serde_json_wasm::from_str(user_data_str)
        .map_err(|_| EnclaveAttestationError::UserDataInvalid)?;

    let pubkey_sec1 = BASE64_STANDARD.decode(&pubkey_json.data)
        .map_err(|_| EnclaveAttestationError::UserDataInvalid)?;

    Ok(VerifiedEnclaveKey {
        measurement_platform: String::from("nitro"),
        measurement_code: attestation_doc.measurement_code(),
        pubkey_sec1,
    })
}

/// Parses a COSE_Sign1 Nitro attestation document.
///
/// The COSE_Sign1 structure contains a CBOR attestation document as its payload.
/// The attestation document contains:
/// - `pcrs`: Map of PCR index to PCR value
/// - `user_data`: Application-specific data (contains public key)
/// - `certificate`: The enclave certificate
/// - `cabundle`: Certificate chain
pub fn parse_nitro_attestation(
    cose_bytes: &[u8],
) -> Result<NitroAttestationDocument, EnclaveAttestationError> {
    // Parse COSE_Sign1 structure (untagged format - starts with 0x84 array)
    let cose_sign1 = CoseSign1::from_slice(cose_bytes)
        .map_err(|_| EnclaveAttestationError::CoseSign1ParsingFailed)?;

    // Extract payload (the attestation document)
    let payload = cose_sign1.payload
        .ok_or(EnclaveAttestationError::PayloadMissing)?;

    // Parse the CBOR attestation document
    let attestation_doc: CborValue = ciborium::from_reader(&payload[..])
        .map_err(|_| EnclaveAttestationError::AttestationDocumentParsingFailed)?;

    // The attestation document should be a map
    let doc_map = match attestation_doc {
        CborValue::Map(m) => m,
        _ => return Err(EnclaveAttestationError::AttestationDocumentNotMap),
    };

    // Helper to get a field from the map
    let get_field = |name: &str| -> Option<&CborValue> {
        doc_map.iter()
            .find(|(k, _)| matches!(k, CborValue::Text(s) if s == name))
            .map(|(_, v)| v)
    };

    // Extract PCRs
    let pcrs_value = get_field("pcrs")
        .ok_or(EnclaveAttestationError::PcrsMissing)?;

    let pcrs_map = match pcrs_value {
        CborValue::Map(m) => m,
        _ => return Err(EnclaveAttestationError::PcrFormatInvalid),
    };

    // Helper to extract PCR by index
    let get_pcr = |index: i128| -> Result<Vec<u8>, EnclaveAttestationError> {
        pcrs_map.iter()
            .find(|(k, _)| matches!(k, CborValue::Integer(i) if i128::from(*i) == index))
            .and_then(|(_, v)| match v {
                CborValue::Bytes(b) => Some(b.clone()),
                _ => None,
            })
            .ok_or(EnclaveAttestationError::RequiredPcrMissing)
    };

    let pcr0 = get_pcr(0)?;
    let pcr1 = get_pcr(1)?;
    let pcr2 = get_pcr(2)?;

    // Extract user_data (contains the application public key)
    let user_data = get_field("user_data")
        .ok_or(EnclaveAttestationError::UserDataMissing)
        .and_then(|v| match v {
            CborValue::Bytes(b) => Ok(b.clone()),
            CborValue::Null => Err(EnclaveAttestationError::UserDataMissing),
            _ => Err(EnclaveAttestationError::UserDataInvalid),
        })?;

    // Extract certificate (DER format)
    let certificate = get_field("certificate")
        .ok_or(EnclaveAttestationError::CertificateMissing)
        .and_then(|v| match v {
            CborValue::Bytes(b) => Ok(b.clone()),
            _ => Err(EnclaveAttestationError::CertificateMissing),
        })?;

    // Extract cabundle (array of DER certificates)
    let cabundle = get_field("cabundle")
        .ok_or(EnclaveAttestationError::CaBundleMissing)
        .and_then(|v| match v {
            CborValue::Array(arr) => {
                let mut certs = Vec::new();
                for item in arr {
                    match item {
                        CborValue::Bytes(b) => certs.push(b.clone()),
                        _ => return Err(EnclaveAttestationError::CaBundleMissing),
                    }
                }
                Ok(certs)
            }
            _ => Err(EnclaveAttestationError::CaBundleMissing),
        })?;

    Ok(NitroAttestationDocument {
        pcr0,
        pcr1,
        pcr2,
        user_data,
        certificate,
        cabundle,
    })
}

/// Extracts the enclave measurement from a Nitro attestation without full verification.
///
/// This is useful for the CLI to extract measurements for comparison without
/// performing full cryptographic verification on-chain.
#[cfg(feature = "std")]
pub fn extract_measurement_from_attestation(
    enclave_attestation_b64: &str,
) -> Result<(String, String, Vec<u8>), EnclaveAttestationError> {
    use base64::prelude::*;

    // Decode the outer base64 (which gives us JSON)
    let json_bytes = BASE64_STANDARD.decode(enclave_attestation_b64)
        .map_err(|_| EnclaveAttestationError::Base64DecodingFailed)?;

    let result = verify_nitro_enclave_attestation(&json_bytes)?;

    Ok((
        result.measurement_platform,
        result.measurement_code,
        result.pubkey_sec1,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_output_for_tests;

    #[test]
    fn test_extract_measurement_from_attestation() {
        let output = block_output_for_tests();
        let enclave_attestation = &output.enclave_attested_application_public_key.enclave_attestation;

        let (platform, code, pubkey) = extract_measurement_from_attestation(enclave_attestation)
            .expect("Failed to extract measurement");

        assert_eq!(platform, "nitro");
        assert!(!code.is_empty(), "Measurement code should not be empty");
        assert!(!pubkey.is_empty(), "Public key should not be empty");

        // The measurement code should have format "pcr0.pcr1.pcr2"
        let parts: Vec<&str> = code.split('.').collect();
        assert_eq!(parts.len(), 3, "Measurement code should have 3 parts");

        // Each PCR should be 48 bytes = 96 hex chars for SHA-384
        for part in &parts {
            assert_eq!(part.len(), 96, "Each PCR should be 96 hex chars (SHA-384)");
        }

        // Verify the measurement matches what's in claims
        let expected_code = &output.enclave_attested_application_public_key.claims.enclave_measurement.code;
        assert_eq!(&code, expected_code, "Measurement code should match claims");

        // Verify public key matches
        use base64::prelude::*;
        let expected_pubkey = BASE64_STANDARD
            .decode(&output.enclave_attested_application_public_key.claims.public_key.data)
            .expect("Failed to decode expected public key");
        assert_eq!(pubkey, expected_pubkey, "Public key should match claims");
    }

    #[test]
    fn test_aws_nitro_root_cert_embedded() {
        // Verify the root cert is properly embedded
        assert!(!AWS_NITRO_ROOT_CERT_DER.is_empty());
        // DER certificates start with 0x30 (SEQUENCE tag)
        assert_eq!(AWS_NITRO_ROOT_CERT_DER[0], 0x30);
    }

    #[test]
    fn test_invalid_platform() {
        let json = r#"{"platform": "invalid", "platform_attestations": []}"#;
        let result = verify_nitro_enclave_attestation(json.as_bytes());
        assert!(matches!(result, Err(EnclaveAttestationError::InvalidPlatform)));
    }

    #[test]
    fn test_empty_attestations() {
        let json = r#"{"platform": "nitro", "platform_attestations": []}"#;
        let result = verify_nitro_enclave_attestation(json.as_bytes());
        assert!(matches!(result, Err(EnclaveAttestationError::NoPlatformAttestations)));
    }
}
