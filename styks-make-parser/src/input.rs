#[derive(serde::Serialize, Debug)]
pub struct NoncePayload {
    nonce: String,
}

impl From<u64> for NoncePayload {
    fn from(nonce: u64) -> Self {
        NoncePayload {
            nonce: nonce.to_string(),
        }
    }
}

#[cfg(feature = "std")]
impl<'a> From<&'a crate::output::SignedPriceData> for crate::Payload {
    fn from(price_data: &'a crate::output::SignedPriceData) -> Self {
        use base64::{Engine, prelude::BASE64_STANDARD};

        crate::Payload {
            path: price_data.request.clone(),
            body: BASE64_STANDARD.encode(price_data.response_body.as_bytes()),
            nonce: price_data.nonce.clone(),
        }
    }
}
