#[cfg(not(feature = "std"))]
use alloc::{format, string::{String, ToString}, vec::Vec, boxed::Box};

use ethabi::{decode, ParamType, Token};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub enum BlockyClaimsError {
    TADataDecoding,
    TADataInvalidLength,
    BytesConversionError,
    OutputJsonDecoding,
    OutputHasNoSuccessStatus,
}

pub struct BlockyClaims {
    pub hash_of_code: Vec<u8>,
    pub function: Vec<u8>,
    pub hash_of_input: Vec<u8>,
    pub output: Vec<u8>,
    pub hash_of_secrets: Vec<u8>,
}

impl BlockyClaims {
    pub fn decode_fn_call_claims(bytes: &[u8]) -> Result<BlockyClaims, BlockyClaimsError> {
        let decoded = decode(&[ParamType::Array(Box::new(ParamType::Bytes))], bytes)
            .map_err(|_| BlockyClaimsError::TADataDecoding)?
            .pop()
            .and_then(|t| t.into_array())
            .ok_or(BlockyClaimsError::TADataDecoding)?;

        if decoded.len() != 5 {
            return Err(BlockyClaimsError::TADataInvalidLength);
        }

        fn extract(data: &Token) -> Result<Vec<u8>, BlockyClaimsError> {
            data
                .clone()
                .into_bytes()
                .ok_or(BlockyClaimsError::BytesConversionError)
        }

        let claims = BlockyClaims {
            hash_of_code: extract(&decoded[0])?,
            function: extract(&decoded[1])?,
            hash_of_input: extract(&decoded[2])?,
            output: extract(&decoded[3])?,
            hash_of_secrets: extract(&decoded[4])?,
        };

        Ok(claims)
    } 

    pub fn hash_of_code(&self) -> String {
        String::from_utf8_lossy(&self.hash_of_code).to_string()
    }

    pub fn function(&self) -> String {
        String::from_utf8_lossy(&self.function).to_string()
    }

    pub fn output_str(&self) -> String {
        String::from_utf8_lossy(&self.output).to_string()
    }

    pub fn output(&self) -> Result<GuestProgramOutputValue, BlockyClaimsError> {
        let output = GuestProgramOutput::try_from_string(&self.output_str())?;
            
        if !output.success {
            return Err(BlockyClaimsError::OutputHasNoSuccessStatus);
        }
        Ok(output.value)
    }

}

#[derive(Deserialize)]
pub struct GuestProgramOutput {
    success: bool,
    error: String,
    value: GuestProgramOutputValue,
}

impl GuestProgramOutput {
    pub fn try_from_string(s: &str) -> Result<Self, BlockyClaimsError> {
        serde_json_wasm::from_str(s)
            .map_err(|_| BlockyClaimsError::OutputJsonDecoding)
    }

    pub fn error_message(&self) -> &str {
        &self.error
    }
}

#[derive(Deserialize)]
pub struct GuestProgramOutputValue {
    pub market: String,
    pub coin_id: String,
    pub currency: String,
    pub price: u64,
    pub timestamp: u64,
}

impl GuestProgramOutputValue {
    pub fn identifier(&self) -> String {
        format!("{}_{}_{}", self.market, self.coin_id, self.currency)
    }
}

#[cfg(test)]
mod tests {
    use crate::{block_output_for_tests, wasm_hash_for_tests};

    use super::*;

    #[test]
    fn test_decode_fn_call_claims() {
        let output = block_output_for_tests();
        let ta = output.ta();
        let data = ta.data();

        let claims = BlockyClaims::decode_fn_call_claims(&data)
            .expect("Failed to decode function call claims");

        // Verify hash of guest code.
        let expected_hash = wasm_hash_for_tests();
        assert_eq!(claims.hash_of_code(), expected_hash);

        // Verify function name.
        assert_eq!(claims.function(), "priceFunc");

        // Verify output.
        let output = claims.output().expect("Failed to get output");
        assert_eq!(output.market, "Gate");
        assert_eq!(output.coin_id, "CSPR");
        assert_eq!(output.currency, "USD");
        assert_eq!(output.price, 516);
        assert_eq!(output.timestamp, 1765796826);
        assert_eq!(output.identifier(), "Gate_CSPR_USD");
    }
}
