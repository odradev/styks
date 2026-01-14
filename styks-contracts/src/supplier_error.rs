use odra::prelude::*;
use styks_blocky_parser::{blocky_claims::BlockyClaimsError, verify::VerificationError};

#[odra::odra_error]
pub enum StyksSupplierError {
    // Config errors.
    ConfigNotSet = 46000,
    PriceFeedIdNotFound = 46001,

    // Role errors.
    NotAdminRole = 46100,
    NotConfigManagerRole = 46101,

    // Verification errors.
    InvalidPublicKey = 46200,
    InvalidSignature = 46201,
    HashingError = 46202,
    BadSignature = 46203,
    BadWasmHash = 46204,
    TimestampOutOfRange = 46205,

    // Claims errors.
    TADataDecoding = 46300,
    TADataInvalidLength = 46301,
    BytesConversionError = 46302,
    OutputJsonDecoding = 46303,
    OutputHasNoSuccessStatus = 46304,

    // Parsing errors.
    InvalidNumberFormat = 46400,
    FieldExtractionError = 46401,
    InvalidTimestamp = 46402,
    DeserializationError = 46403,
}

impl From<styks_make_parser::ParsingError> for StyksSupplierError {
    fn from(e: styks_make_parser::ParsingError) -> Self {
        match e {
            styks_make_parser::ParsingError::InvalidNumberFormat => {
                StyksSupplierError::InvalidNumberFormat
            }
            styks_make_parser::ParsingError::FieldExtractionError => {
                StyksSupplierError::FieldExtractionError
            }
            styks_make_parser::ParsingError::InvalidTimestamp => {
                StyksSupplierError::InvalidTimestamp
            }
            styks_make_parser::ParsingError::DeserializationError => {
                StyksSupplierError::DeserializationError
            }
        }
    }
}

impl From<VerificationError> for StyksSupplierError {
    fn from(error: VerificationError) -> Self {
        match error {
            VerificationError::InvalidPublicKey => StyksSupplierError::InvalidPublicKey,
            VerificationError::InvalidSignature => StyksSupplierError::InvalidSignature,
            VerificationError::HashingError => StyksSupplierError::HashingError,
            VerificationError::BadSignature => StyksSupplierError::BadSignature,
        }
    }
}

impl From<BlockyClaimsError> for StyksSupplierError {
    fn from(error: BlockyClaimsError) -> Self {
        match error {
            BlockyClaimsError::TADataDecoding => StyksSupplierError::TADataDecoding,
            BlockyClaimsError::TADataInvalidLength => StyksSupplierError::TADataInvalidLength,
            BlockyClaimsError::BytesConversionError => StyksSupplierError::BytesConversionError,
            BlockyClaimsError::OutputJsonDecoding => StyksSupplierError::OutputJsonDecoding,
            BlockyClaimsError::OutputHasNoSuccessStatus => {
                StyksSupplierError::OutputHasNoSuccessStatus
            }
        }
    }
}
