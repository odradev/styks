use crate::output::SignedPriceData;

pub fn test_data() -> SignedPriceData {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(manifest_dir).join("../resources/test/make_out.json");
    SignedPriceData::try_from_file(path).expect("Failed to load SignedPriceData")
}
