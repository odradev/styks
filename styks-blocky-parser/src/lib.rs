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

#[cfg(test)]
fn block_output_for_tests() -> blocky_output::BlockyOutput {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(manifest_dir).join("../resources/test/1_out.json");
    blocky_output::BlockyOutput::try_from_file(path).expect("Failed to load BlockyOutput")
}

#[cfg(test)]
fn wasm_hash_for_tests() -> String {
    let wasm = include_bytes!("../../resources/test/1_guest.wasm");
    wasm_hash(wasm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_hash() {
        let expected_hash = "88b33f65cab461748c49de1edb5f81974dcc98a171d223c537b8a6869c348f3497daa7aba61654a5c4e1813171ed4238a079fac1f75d3c817bc152aa60d4c2e0";
        assert_eq!(wasm_hash_for_tests(), expected_hash);
    }
}