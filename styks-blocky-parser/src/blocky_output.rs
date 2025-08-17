use std::{fs, path::Path};

use base64::{prelude::BASE64_STANDARD, Engine};
use ethabi::{decode, ParamType};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey, signature::hazmat::PrehashVerifier};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};


pub type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockyOutput {
    pub enclave_attested_application_public_key: EnclaveAttestedApplicationPublicKey,
    pub transitive_attested_function_call: TransitiveAttestedFunctionCall,
}

impl BlockyOutput {
    pub fn try_from_file<P: AsRef<Path>>(path: P) -> Result<Self, DynError> {
        let text = fs::read_to_string(path)?;
        let parsed: BlockyOutput = serde_json::from_str(&text)?;
        Ok(parsed)
    }

    pub fn public_key(&self) -> VerifyingKey {
        let public_key_str = &self.enclave_attested_application_public_key.claims.public_key.data;
        let public_key_bytes = BASE64_STANDARD.decode(public_key_str).unwrap();
        let public_key = VerifyingKey::from_sec1_bytes(&public_key_bytes)
            .expect("Failed to parse original public key from SEC1 bytes");
        public_key
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key().to_sec1_bytes().to_vec()
    }

    pub fn ta(&self) -> TA {
        let ta_data = &self.transitive_attested_function_call.transitive_attestation;
        let ta_data = BASE64_STANDARD.decode(ta_data).expect("Failed to decode TA data");
        TA::new(&ta_data)
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveAttestedApplicationPublicKey {
    pub enclave_attestation: String,
    pub claims: EnclaveAttestedApplicationPublicKeyClaims,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveAttestedApplicationPublicKeyClaims {
    pub enclave_measurement: EnclaveMeasurement,
    pub public_key: PublicKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveMeasurement {
    pub platform: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub curve_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitiveAttestedFunctionCall {
    pub transitive_attestation: String,
    pub claims: TransitiveClaims,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitiveClaims {
    pub hash_of_code: String,
    pub function: String,
    pub hash_of_input: String,
    // This field is a JSON string; use helper below to parse it if needed.
    pub output: String,
    pub hash_of_secrets: String,
}

impl TransitiveClaims {
    pub fn parse_output_as_value(&self) -> Result<serde_json::Value, DynError> {
        let v: serde_json::Value = serde_json::from_str(&self.output)?;
        Ok(v)
    }

    pub fn parse_output_as_typed(&self) -> Result<FunctionOutput, DynError> {
        let v: FunctionOutput = serde_json::from_str(&self.output)?;
        Ok(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionOutput {
    pub success: bool,
    pub error: String,
    pub value: FunctionOutputValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionOutputValue {
    pub market: String,
    pub coin_id: String,
    pub currency: String,
    pub price: f64,
    pub timestamp: String,
}

pub struct TA {
    data: Vec<u8>,
    signature: Signature,
    recovery_id: RecoveryId,
}

impl TA {
    pub fn new(bytes: &[u8]) -> Self {
        let decoded = decode(&[ParamType::Array(Box::new(ParamType::Bytes))], bytes)
            .expect("Failed to decode TA data");

        let bytes_array = decoded[0].clone().into_array().expect("Expected array");
        if bytes_array.len() != 2 {
            panic!("Expected 2 elements in TA data, got {}", bytes_array.len());
        }

        let data = bytes_array[0]
            .clone()
            .into_bytes()
            .expect("Expected bytes for data");
        let sig_bytes = bytes_array[1]
            .clone()
            .into_bytes()
            .expect("Expected bytes for signature");

        if sig_bytes.len() < 65 {
            panic!(
                "Signature too short, expected 65 bytes (r + s + v), got {}",
                sig_bytes.len()
            );
        }

        // The k256::Signature type is just r and s (64 bytes)
        let signature = Signature::from_slice(&sig_bytes[..64])
            .expect("Failed to create signature from r and s");

        // The recovery ID is the last byte
        let recovery_id = RecoveryId::from_byte(sig_bytes[64]).expect("Invalid recovery ID byte");

        TA {
            data,
            signature,
            recovery_id,
        }
    }

    // This function recovers the public key
    pub fn recover_public_key(&self) -> VerifyingKey {
        // Hash the data using Keccak256, matching Solidity's keccak256()
        let mut hasher = Keccak256::new();
        hasher.update(&self.data);
        let data_hash = hasher.finalize();

        // Recover the verifying key (public key) from the hash and signature
        VerifyingKey::recover_from_prehash(&data_hash, &self.signature, self.recovery_id)
            .expect("Failed to recover public key")

        // // I wish this worked:
        // VerifyingKey::recover_from_msg(&self.data, &self.signature, self.recovery_id)
        //     .expect("Failed to recover public key")
    }

    // This function verifies the signature against the data.
    pub fn verify_signature(&self, pk: &VerifyingKey) -> bool {
        // Hash the data using Keccak256, matching Solidity's keccak256()
        let mut hasher = Keccak256::new();
        hasher.update(&self.data);
        let data_hash = hasher.finalize();

        // Verify the signature against the prehashed data
        pk.verify_prehash(&data_hash, &self.signature).is_ok()
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn signature_bytes(&self) -> Vec<u8> {
        self.signature().to_vec()
    }


    pub fn data(&self) -> &[u8] {
        &self.data
    }
}



#[cfg(test)]
mod tests {
    use crate::block_output_for_tests;

    #[test]
    fn test_load_blocky_output_from_file() {
        let output = block_output_for_tests();
        assert!(output.enclave_attested_application_public_key.enclave_attestation.len() > 0);
        assert!(output.transitive_attested_function_call.transitive_attestation.len() > 0);
    }
}
