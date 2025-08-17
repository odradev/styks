use k256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};
use sha3::{Digest, Keccak256};

#[derive(Debug, Clone, PartialEq)]
pub enum VerificationError {
    InvalidPublicKey,
    InvalidSignature,
    HashingError,
    BadSignature,
}

pub fn verify_signature(
    public_key: &[u8],
    signature: &[u8],
    data: &[u8],
) -> Result<(), VerificationError> {
    // Parse public key.
    let public_key = VerifyingKey::from_sec1_bytes(public_key)
        .map_err(|_| VerificationError::InvalidPublicKey)?;

    // Parse signature.
    let signature = Signature::from_slice(&signature)
        .map_err(|_| VerificationError::InvalidSignature)?;

    // Hash the data.
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let data_hash = hasher.finalize();

    public_key
        .verify_prehash(&data_hash, &signature)
        .map_err(|_| VerificationError::BadSignature)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{block_output_for_tests};

    use super::*;

    #[test]
    fn test_verify_signature() {
        let output = block_output_for_tests();
        let ta = output.ta();

        // Test TA verification.
        assert!(ta.verify_signature(&output.public_key()));

        // Test standalone verification.
        let public_key = output.public_key_bytes();
        let signature = ta.signature_bytes();
        let data = ta.data();
        assert!(verify_signature(&public_key, &signature, &data).is_ok());

        // Test with invalid public key.
        let invalid_public_key = [0u8; 65]; // Invalid public key length
        assert_eq!(
            verify_signature(&invalid_public_key, &signature, &data),
            Err(VerificationError::InvalidPublicKey)
        );

        // Test with invalid signature.
        let invalid_signature = [0u8; 64]; // Invalid signature length
        assert_eq!(
            verify_signature(&public_key, &invalid_signature, &data),
            Err(VerificationError::InvalidSignature)
        );

        // Test with bad signature.
        let bad_signature = [1u8; 64]; // Arbitrary bytes that do not form a valid signature
        assert_eq!(
            verify_signature(&public_key, &bad_signature, &data),
            Err(VerificationError::BadSignature)
        );

        // Test with different data.
        let different_data = b"Different data";
        assert_eq!(
            verify_signature(&public_key, &signature, different_data),
            Err(VerificationError::BadSignature)
        );
    }
}