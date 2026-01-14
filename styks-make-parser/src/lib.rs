#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
pub mod input;
#[cfg(feature = "std")]
pub mod output;
#[cfg(feature = "test-utils")]
pub mod test_utils;

mod parser;
use base64::{Engine, prelude::BASE64_STANDARD};
pub use parser::{ParsedData, ParsingError, parse_data};

#[derive(serde::Deserialize, Debug)]
pub struct Data<T> {
    pub data: T,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Payload {
    path: String,
    body: String,
    nonce: String,
}

impl Payload {
    pub fn new(path: String, body: String, nonce: String) -> Self {
        Self { path, body, nonce }
    }

    pub fn decoded_body(&self) -> Option<String> {
        BASE64_STANDARD
            .decode(&self.body)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }
}
