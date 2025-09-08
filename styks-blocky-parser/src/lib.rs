#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
use sha3::{Digest, Sha3_512};

pub mod blocky_claims;

#[cfg(feature = "std")]
pub mod blocky_output;
pub mod verify;

#[cfg(feature = "std")]
pub fn wasm_hash(wasm_bytes: &[u8]) -> String {
    let mut hasher = Sha3_512::new();
    hasher.update(wasm_bytes);
    hex::encode(hasher.finalize())
}

#[cfg(feature = "std")]
pub fn block_output_for_tests() -> blocky_output::BlockyOutput {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(manifest_dir).join("../resources/test/2_out.json");
    blocky_output::BlockyOutput::try_from_file(path).expect("Failed to load BlockyOutput")
}

#[cfg(feature = "std")]
pub fn wasm_hash_for_tests() -> String {
    let wasm = include_bytes!("../../resources/test/1_guest.wasm");
    wasm_hash(wasm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_hash() {
        let expected_hash = "baadaf829374304416a3c78a7c1118eb6784d3585c8cb5b18fa95c38cb8e4382fda8e149c4d05769d513af599445237dcc87d232da8f51251f0ad6dd1aff5b17";
        assert_eq!(wasm_hash_for_tests(), expected_hash);
    }
}