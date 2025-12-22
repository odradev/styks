//! StyksBlockySupplier - Measurement-Anchored Model
//!
//! This contract bridges Blocky attestation service to StyksPriceFeed.
//! It uses a measurement-anchored model where signer keys are accepted only
//! when proven via verified AWS Nitro attestation matching an on-chain
//! measurement allowlist.

use odra::{casper_types::bytesrepr::Bytes, prelude::*, ContractRef};
use odra_modules::access::{AccessControl, Role, DEFAULT_ADMIN_ROLE};
use styks_blocky_parser::{
    blocky_claims::{BlockyClaims, BlockyClaimsError},
    transitive_attestation::{decode_transitive_attestation, TransitiveAttestationError},
    verify::VerificationError,
};
use styks_core::{Price, PriceFeedId};

use crate::styks_price_feed::StyksPriceFeedContractRef;

// --- Errors ---

#[odra::odra_error]
pub enum StyksBlockySupplierError {
    // Config errors.
    ConfigNotSet = 46000,
    PriceFeedIdNotFound = 46001,

    // Role errors.
    NotAdminRole = 46100,
    NotConfigManagerRole = 46101,
    NotGuardianRole = 46102,

    // Verification errors.
    InvalidPublicKey = 46200,
    InvalidSignature = 46201,
    HashingError = 46202,
    BadSignature = 46203,
    BadWasmHash = 46204,
    TimestampOutOfRange = 46205,
    BadFunctionName = 46206,

    // Claims errors.
    TADataDecoding = 46300,
    TADataInvalidLength = 46301,
    BytesConversionError = 46302,
    OutputJsonDecoding = 46303,
    OutputHasNoSuccessStatus = 46304,

    // Transitive attestation decoding errors.
    TAAbiDecodingFailed = 46350,
    TAInvalidArrayLength = 46351,
    TAMissingDataElement = 46352,
    TAMissingSignatureElement = 46353,
    TASignatureTooShort = 46354,

    // Enclave attestation errors.
    EnclaveAttestationFailed = 46400,
    MeasurementNotAllowed = 46401,
    SignerNotFound = 46402,
    SignerRevoked = 46403,
    SignerExpired = 46404,
    OnChainAttestationVerificationDisabled = 46405,

    // Replay protection.
    ReplayDetected = 46500,

    // Contract state.
    ContractPaused = 46600,

    // Argument errors.
    MissingSignerResolution = 46700,
}

impl From<VerificationError> for StyksBlockySupplierError {
    fn from(error: VerificationError) -> Self {
        use VerificationError::*;
        match error {
            InvalidPublicKey => StyksBlockySupplierError::InvalidPublicKey,
            InvalidSignature => StyksBlockySupplierError::InvalidSignature,
            HashingError => StyksBlockySupplierError::HashingError,
            BadSignature => StyksBlockySupplierError::BadSignature,
        }
    }
}

impl From<BlockyClaimsError> for StyksBlockySupplierError {
    fn from(error: BlockyClaimsError) -> Self {
        use BlockyClaimsError::*;
        match error {
            TADataDecoding => StyksBlockySupplierError::TADataDecoding,
            TADataInvalidLength => StyksBlockySupplierError::TADataInvalidLength,
            BytesConversionError => StyksBlockySupplierError::BytesConversionError,
            OutputJsonDecoding => StyksBlockySupplierError::OutputJsonDecoding,
            OutputHasNoSuccessStatus => StyksBlockySupplierError::OutputHasNoSuccessStatus,
        }
    }
}

impl From<TransitiveAttestationError> for StyksBlockySupplierError {
    fn from(error: TransitiveAttestationError) -> Self {
        use TransitiveAttestationError::*;
        match error {
            AbiDecodingFailed => StyksBlockySupplierError::TAAbiDecodingFailed,
            InvalidArrayLength => StyksBlockySupplierError::TAInvalidArrayLength,
            MissingDataElement => StyksBlockySupplierError::TAMissingDataElement,
            MissingSignatureElement => StyksBlockySupplierError::TAMissingSignatureElement,
            SignatureTooShort => StyksBlockySupplierError::TASignatureTooShort,
        }
    }
}

// --- Access Control Roles ---

#[derive(Debug)]
pub enum StyksBlockySupplierRole {
    Admin,
    ConfigManager,
    Guardian,
}

impl StyksBlockySupplierRole {
    pub fn role_id(&self) -> Role {
        match self {
            StyksBlockySupplierRole::Admin => DEFAULT_ADMIN_ROLE,
            // Start with 3, so it doesn't overlap with PriceFeed.
            StyksBlockySupplierRole::ConfigManager => [3u8; 32],
            StyksBlockySupplierRole::Guardian => [4u8; 32],
        }
    }
}

// --- Configuration ---

/// A measurement rule that defines an allowed enclave measurement.
#[odra::odra_type]
pub struct MeasurementRule {
    /// The platform identifier (e.g., "nitro").
    pub platform: String,
    /// The measurement code (e.g., "pcr0hex.pcr1hex.pcr2hex").
    pub code: String,
}

/// Configuration for the StyksBlockySupplier contract.
#[odra::odra_type]
pub struct StyksBlockySupplierConfig {
    /// The expected WASM hash of the guest program.
    pub wasm_hash: String,
    /// The expected function name in the guest program (e.g., "priceFunc").
    pub expected_function: String,
    /// List of allowed enclave measurements.
    pub allowed_measurements: Vec<MeasurementRule>,
    /// Mapping from CoinGecko identifiers to PriceFeedIds.
    pub coingecko_feed_ids: Vec<(String, PriceFeedId)>,
    /// Address of the StyksPriceFeed contract.
    pub price_feed_address: Address,
    /// Tolerance in seconds for timestamp validation.
    pub timestamp_tolerance: u64,
    /// Time-to-live for cached signers in seconds. 0 disables TTL.
    pub signer_ttl_secs: u64,
}

impl StyksBlockySupplierConfig {
    /// Returns the PriceFeedId for a given CoinGecko identifier.
    pub fn price_feed_id(&self, coingecko_id: &str) -> Option<PriceFeedId> {
        self.coingecko_feed_ids
            .iter()
            .find(|(id, _)| id == coingecko_id)
            .map(|(_, feed_id)| feed_id.clone())
    }

    /// Checks if a measurement is in the allowed list.
    pub fn is_measurement_allowed(&self, platform: &str, code: &str) -> bool {
        self.allowed_measurements
            .iter()
            .any(|rule| rule.platform == platform && rule.code == code)
    }
}

// --- Cached Signer ---

/// A cached signer entry storing a verified enclave key.
#[odra::odra_type]
pub struct CachedSigner {
    /// The SEC1-encoded public key (65 bytes for secp256k1 uncompressed).
    pub pubkey_sec1: Bytes,
    /// The measurement platform (e.g., "nitro").
    pub measurement_platform: String,
    /// The measurement code (e.g., "pcr0hex.pcr1hex.pcr2hex").
    pub measurement_code: String,
    /// Timestamp when this signer was registered.
    pub registered_at: u64,
    /// Timestamp when this signer last submitted a price.
    pub last_seen: u64,
    /// Whether this signer has been revoked.
    pub revoked: bool,
}

// --- StyksBlockySupplier Contract ---

#[odra::module]
pub struct StyksBlockySupplier {
    access_control: SubModule<AccessControl>,
    config: Var<StyksBlockySupplierConfig>,
    /// Mapping from signer_id (keccak256 of pubkey) to CachedSigner.
    cached_signers: Mapping<Bytes, CachedSigner>,
    /// Last accepted timestamp per feed for replay protection.
    last_accepted_timestamp: Mapping<PriceFeedId, u64>,
    /// Whether the contract is paused.
    paused: Var<bool>,
}

#[odra::module]
impl StyksBlockySupplier {
    pub fn init(&mut self) {
        // Grant the admin role to the contract deployer.
        let deployer = self.env().caller();
        let admin_role = StyksBlockySupplierRole::Admin.role_id();
        self.access_control
            .unchecked_grant_role(&admin_role, &deployer);
        self.paused.set(false);
    }

    delegate! {
        to self.access_control {
            fn has_role(&self, role: &Role, address: &Address) -> bool;
            fn grant_role(&mut self, role: &Role, address: &Address);
            fn revoke_role(&mut self, role: &Role, address: &Address);
            fn get_role_admin(&self, role: &Role) -> Role;
            fn renounce_role(&mut self, role: &Role, address: &Address);
        }
    }

    // --- Config Management ---

    pub fn set_config(&mut self, config: StyksBlockySupplierConfig) {
        // Make sure only ConfigManager can set the config.
        self.assert_config_manager(&self.env().caller());

        // Update the config.
        self.config.set(config);
    }

    pub fn get_config(&self) -> StyksBlockySupplierConfig {
        self.config
            .get()
            .unwrap_or_revert_with(&self.env(), StyksBlockySupplierError::ConfigNotSet)
    }

    pub fn get_config_or_none(&self) -> Option<StyksBlockySupplierConfig> {
        self.config.get()
    }

    // --- Pause/Unpause ---

    /// Pauses the contract. Only Guardian or Admin can pause.
    pub fn pause(&mut self) {
        self.assert_guardian_or_admin(&self.env().caller());
        self.paused.set(true);
    }

    /// Unpauses the contract. Only Guardian or Admin can unpause.
    pub fn unpause(&mut self) {
        self.assert_guardian_or_admin(&self.env().caller());
        self.paused.set(false);
    }

    /// Returns whether the contract is paused.
    pub fn is_paused(&self) -> bool {
        self.paused.get().unwrap_or(false)
    }

    // --- Signer Management ---

    /// Revokes a signer. Only Guardian or Admin can revoke.
    pub fn revoke_signer(&mut self, signer_id: Bytes) {
        self.assert_guardian_or_admin(&self.env().caller());

        if let Some(mut signer) = self.cached_signers.get(&signer_id) {
            signer.revoked = true;
            self.cached_signers.set(&signer_id, signer);
        }
        // Silently ignore if signer doesn't exist
    }

    /// Returns a cached signer by ID, if it exists.
    pub fn get_signer(&self, signer_id: Bytes) -> Option<CachedSigner> {
        self.cached_signers.get(&signer_id)
    }

    /// Registers a new signer via full on-chain attestation verification.
    ///
    /// This is permissionless when on-chain verification is enabled.
    /// In fallback mode, this reverts with OnChainAttestationVerificationDisabled.
    ///
    /// Returns the signer_id (keccak256 of pubkey).
    pub fn register_signer(&mut self, _enclave_attestation: Bytes) -> Bytes {
        // In fallback mode, on-chain attestation verification is disabled.
        // The CLI must use register_signer_manual instead.
        //
        // TODO: Implement full on-chain COSE + X.509 verification when ready.
        // For now, we operate in fallback mode.
        self.env()
            .revert(StyksBlockySupplierError::OnChainAttestationVerificationDisabled);
    }

    /// Registers a signer manually. Only Guardian or Admin can call this.
    ///
    /// This is used in fallback mode when on-chain attestation verification
    /// is too expensive. The CLI verifies the attestation off-chain and then
    /// calls this function.
    ///
    /// Returns the signer_id (keccak256 of pubkey).
    pub fn register_signer_manual(
        &mut self,
        pubkey_sec1: Bytes,
        measurement_platform: String,
        measurement_code: String,
    ) -> Bytes {
        self.assert_guardian_or_admin(&self.env().caller());

        let config = self.get_config();

        // Verify the measurement is in the allowlist.
        if !config.is_measurement_allowed(&measurement_platform, &measurement_code) {
            self.env()
                .revert(StyksBlockySupplierError::MeasurementNotAllowed);
        }

        // Compute signer_id as keccak256(pubkey_sec1).
        let signer_id = self.compute_signer_id(&pubkey_sec1);

        let now = self.env().get_block_time_secs();

        // Create or update the cached signer.
        let signer = CachedSigner {
            pubkey_sec1: pubkey_sec1.clone(),
            measurement_platform,
            measurement_code,
            registered_at: now,
            last_seen: now,
            revoked: false,
        };

        self.cached_signers.set(&signer_id, signer);

        signer_id
    }

    // --- Main Reporting Entrypoint ---

    /// Reports prices from a transitive attestation.
    ///
    /// # Arguments
    ///
    /// * `transitive_attestation` - The base64-decoded transitive attestation blob
    /// * `signer_id` - Optional signer ID for fast path (cached signer lookup)
    /// * `enclave_attestation` - Optional enclave attestation for slow path (on-chain verify + cache)
    ///
    /// At least one of `signer_id` or `enclave_attestation` must be provided.
    pub fn report_prices(
        &mut self,
        transitive_attestation: Bytes,
        signer_id: Option<Bytes>,
        enclave_attestation: Option<Bytes>,
    ) {
        // 1. Check not paused
        if self.is_paused() {
            self.env().revert(StyksBlockySupplierError::ContractPaused);
        }

        let config = self.get_config();
        let now = self.env().get_block_time_secs();

        // 2. Decode transitive attestation
        let decoded_ta = match decode_transitive_attestation(&transitive_attestation) {
            Ok(ta) => ta,
            Err(e) => self.env().revert(StyksBlockySupplierError::from(e)),
        };

        let data = decoded_ta.data;
        let signature_rs = decoded_ta.signature_rs;

        // 3. Resolve signer
        let (resolved_signer_id, pubkey) = if let Some(sid) = signer_id {
            // Fast path: lookup cached signer
            let signer = self
                .cached_signers
                .get(&sid)
                .unwrap_or_else(|| self.env().revert(StyksBlockySupplierError::SignerNotFound));

            // Check not revoked
            if signer.revoked {
                self.env().revert(StyksBlockySupplierError::SignerRevoked);
            }

            // Check not expired (if TTL is enabled)
            if config.signer_ttl_secs > 0 {
                let expiry = signer.registered_at.saturating_add(config.signer_ttl_secs);
                if now > expiry {
                    self.env().revert(StyksBlockySupplierError::SignerExpired);
                }
            }

            (sid, signer.pubkey_sec1.to_vec())
        } else if let Some(_attestation) = enclave_attestation {
            // Slow path: on-chain verification (currently disabled)
            // TODO: Implement when on-chain verification is ready
            self.env()
                .revert(StyksBlockySupplierError::OnChainAttestationVerificationDisabled);
        } else {
            // Neither signer_id nor enclave_attestation provided
            self.env()
                .revert(StyksBlockySupplierError::MissingSignerResolution);
        };

        // 4. Verify TA signature
        self.assert_valid_signature(&pubkey, &signature_rs, &data);

        // 5. Decode claims
        let claims = match BlockyClaims::decode_fn_call_claims(&data) {
            Ok(claims) => claims,
            Err(error) => {
                self.env().revert(StyksBlockySupplierError::from(error));
            }
        };

        // 6. Verify WASM hash
        if claims.hash_of_code() != config.wasm_hash {
            self.env().revert(StyksBlockySupplierError::BadWasmHash);
        }

        // 7. Verify function name
        if claims.function() != config.expected_function {
            self.env().revert(StyksBlockySupplierError::BadFunctionName);
        }

        // 8. Extract output
        let output = match claims.output() {
            Ok(output) => output,
            Err(error) => {
                self.env().revert(StyksBlockySupplierError::from(error));
            }
        };

        // 9. Verify timestamp tolerance
        self.assert_timestamp_in_range(output.timestamp, config.timestamp_tolerance);

        // 10. Get PriceFeedId
        let price_feed_id = match config.price_feed_id(&output.identifier()) {
            Some(id) => PriceFeedId::from(id),
            None => self
                .env()
                .revert(StyksBlockySupplierError::PriceFeedIdNotFound),
        };

        // 11. Replay protection
        let last_ts = self.last_accepted_timestamp.get(&price_feed_id).unwrap_or(0);
        if output.timestamp <= last_ts {
            self.env().revert(StyksBlockySupplierError::ReplayDetected);
        }

        // 12. Load the price feed
        let mut feed = StyksPriceFeedContractRef::new(self.env(), config.price_feed_address);

        // 13. Report the price
        let price = Price::from(output.price);
        feed.add_to_feed(vec![(price_feed_id.clone(), price)]);

        // 14. Update last_accepted_timestamp
        self.last_accepted_timestamp.set(&price_feed_id, output.timestamp);

        // 15. Update signer's last_seen
        if let Some(mut signer) = self.cached_signers.get(&resolved_signer_id) {
            signer.last_seen = now;
            self.cached_signers.set(&resolved_signer_id, signer);
        }
    }

    // --- Legacy Entrypoint (deprecated) ---

    /// Legacy entrypoint for backwards compatibility.
    /// Prefer using `report_prices` with signer caching.
    #[deprecated(note = "Use report_prices with signer caching instead")]
    pub fn report_signed_prices(&mut self, _signature: Bytes, _data: Bytes) {
        // This legacy entrypoint is no longer supported in the measurement-anchored model.
        // The old model used a static public key from config, which has been removed.
        // Use report_prices with a registered signer instead.
        self.env()
            .revert(StyksBlockySupplierError::OnChainAttestationVerificationDisabled);
    }
}

impl StyksBlockySupplier {
    fn assert_role(&self, address: &Address, role: StyksBlockySupplierRole) {
        if !self.has_role(&role.role_id(), address) {
            use StyksBlockySupplierError::*;
            use StyksBlockySupplierRole::*;
            let error = match role {
                Admin => NotAdminRole,
                ConfigManager => NotConfigManagerRole,
                Guardian => NotGuardianRole,
            };
            self.env().revert(error);
        }
    }

    fn assert_config_manager(&self, address: &Address) {
        self.assert_role(address, StyksBlockySupplierRole::ConfigManager);
    }

    fn assert_guardian_or_admin(&self, address: &Address) {
        let is_guardian = self.has_role(&StyksBlockySupplierRole::Guardian.role_id(), address);
        let is_admin = self.has_role(&StyksBlockySupplierRole::Admin.role_id(), address);

        if !is_guardian && !is_admin {
            self.env()
                .revert(StyksBlockySupplierError::NotGuardianRole);
        }
    }

    fn assert_valid_signature(&self, public_key: &[u8], signature: &[u8], data: &[u8]) {
        let result = styks_blocky_parser::verify::verify_signature(public_key, signature, data);
        if let Err(error) = result {
            self.env().revert(StyksBlockySupplierError::from(error));
        }
    }

    fn assert_timestamp_in_range(&self, reported: u64, tolerance: u64) {
        let current_time = self.env().get_block_time_secs();
        if reported < current_time.saturating_sub(tolerance) || reported > current_time + tolerance
        {
            self.env()
                .revert(StyksBlockySupplierError::TimestampOutOfRange);
        }
    }

    fn compute_signer_id(&self, pubkey_sec1: &Bytes) -> Bytes {
        // Use a simple hash of the pubkey as the signer_id.
        // We use the Casper crypto primitives available in Odra.
        // For now, use blake2b hash which is available in Casper.
        let hash = self.env().hash(pubkey_sec1);
        Bytes::from(hash.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use odra::host::{Deployer, HostEnv, NoArgs};
    use styks_blocky_parser::blocky_output::BlockyOutput;

    use crate::styks_price_feed::{
        StyksPriceFeed, StyksPriceFeedConfig, StyksPriceFeedHostRef, StyksPriceFeedRole,
    };

    use super::*;

    fn setup() -> (
        HostEnv,
        StyksPriceFeedHostRef,
        StyksBlockySupplierHostRef,
        StyksBlockySupplierConfig,
        BlockyOutput,
    ) {
        let env = odra_test::env();
        let admin = env.get_account(0);

        // Load BlockyOutput from file.
        let blocky_output = BlockyOutput::try_from_file("../resources/test/2_out.json")
            .expect("Failed to load BlockyOutput");

        // Load guest wasm bytes.
        let wasm_bytes = include_bytes!("../../resources/test/1_guest.wasm");
        let wasm_hash = styks_blocky_parser::wasm_hash(wasm_bytes);

        // Deploy StyksPriceFeed contract.
        let mut feed = StyksPriceFeed::deploy(&env, NoArgs);
        let feed_config = StyksPriceFeedConfig {
            heartbeat_interval: 100,
            heartbeat_tolerance: 45,
            twap_window: 1,
            twap_tolerance: 0,
            price_feed_ids: vec![String::from("CSPRUSD")],
        };
        feed.grant_role(&StyksPriceFeedRole::ConfigManager.role_id(), &admin);
        feed.set_config(feed_config);

        // Deploy StyksBlockySupplier contract.
        let mut supplier = StyksBlockySupplier::deploy(&env, NoArgs);
        let supplier_config = StyksBlockySupplierConfig {
            wasm_hash,
            expected_function: String::from("priceFunc"),
            allowed_measurements: vec![MeasurementRule {
                platform: String::from("nitro"),
                code: String::from("test_measurement_code"),
            }],
            coingecko_feed_ids: vec![(String::from("Gate_CSPR_USD"), String::from("CSPRUSD"))],
            price_feed_address: feed.address(),
            timestamp_tolerance: 1, // 1 sec tolerance
            signer_ttl_secs: 0,     // Disable TTL for tests
        };
        supplier.grant_role(&StyksBlockySupplierRole::ConfigManager.role_id(), &admin);
        supplier.grant_role(&StyksBlockySupplierRole::Guardian.role_id(), &admin);
        supplier.set_config(supplier_config.clone());

        // Allow StyksBlockySupplier to add prices to StyksPriceFeed.
        let role = StyksPriceFeedRole::PriceSupplier.role_id();
        feed.grant_role(&role, &supplier.address());

        (env, feed, supplier, supplier_config, blocky_output)
    }

    #[test]
    fn test_pause_unpause() {
        let (_env, _feed, mut supplier, _config, _blocky_output) = setup();

        // Initially not paused
        assert!(!supplier.is_paused());

        // Pause
        supplier.pause();
        assert!(supplier.is_paused());

        // Unpause
        supplier.unpause();
        assert!(!supplier.is_paused());
    }

    #[test]
    fn test_register_signer_manual() {
        let (_env, _feed, mut supplier, _config, blocky_output) = setup();

        let pubkey = Bytes::from(blocky_output.public_key_bytes());

        // Register signer manually
        let signer_id = supplier.register_signer_manual(
            pubkey.clone(),
            String::from("nitro"),
            String::from("test_measurement_code"),
        );

        // Verify signer was cached
        let signer = supplier.get_signer(signer_id.clone()).expect("Signer should exist");
        assert_eq!(signer.pubkey_sec1, pubkey);
        assert_eq!(signer.measurement_platform, "nitro");
        assert!(!signer.revoked);
    }

    #[test]
    fn test_revoke_signer() {
        let (_env, _feed, mut supplier, _config, blocky_output) = setup();

        let pubkey = Bytes::from(blocky_output.public_key_bytes());

        // Register signer
        let signer_id = supplier.register_signer_manual(
            pubkey,
            String::from("nitro"),
            String::from("test_measurement_code"),
        );

        // Verify not revoked
        let signer = supplier.get_signer(signer_id.clone()).unwrap();
        assert!(!signer.revoked);

        // Revoke
        supplier.revoke_signer(signer_id.clone());

        // Verify revoked
        let signer = supplier.get_signer(signer_id).unwrap();
        assert!(signer.revoked);
    }

    #[test]
    fn test_report_prices_with_cached_signer() {
        use base64::prelude::*;
        use styks_blocky_parser::blocky_claims::BlockyClaims;

        let (env, feed, mut supplier, config, blocky_output) = setup();
        let id = config.coingecko_feed_ids[0].1.clone();

        // Get the actual measurement from the attestation
        let enclave_att = &blocky_output
            .enclave_attested_application_public_key
            .enclave_attestation;
        let (platform, code, _pubkey) =
            styks_blocky_parser::nitro::extract_measurement_from_attestation(enclave_att)
                .expect("Failed to extract measurement");

        // Update config with actual measurement
        let mut new_config = config.clone();
        new_config.allowed_measurements = vec![MeasurementRule {
            platform: platform.clone(),
            code: code.clone(),
        }];
        new_config.timestamp_tolerance = 20 * 60; // 20 minutes tolerance
        supplier.set_config(new_config);

        // Register signer with actual measurement
        let pubkey = Bytes::from(blocky_output.public_key_bytes());
        let signer_id = supplier.register_signer_manual(pubkey, platform, code);

        // Set block time to match the attestation timestamp (get from TA claims)
        let ta = blocky_output.ta();
        let claims = BlockyClaims::decode_fn_call_claims(ta.data()).expect("decode claims");
        let output = claims.output().expect("get output");
        let timestamp = output.timestamp;
        env.advance_block_time(timestamp * 1000);

        // Price should be empty initially
        assert_eq!(feed.get_twap_price(&id), None);

        // Get transitive attestation bytes
        let ta_b64 = &blocky_output.transitive_attested_function_call.transitive_attestation;
        let ta_bytes = BASE64_STANDARD.decode(ta_b64).expect("Failed to decode TA");

        // Report prices using cached signer (fast path)
        supplier.report_prices(Bytes::from(ta_bytes), Some(signer_id), None);

        // Check the reported price
        let price = feed.get_twap_price(&id);
        assert_eq!(price, Some(516));
    }

    #[test]
    fn test_replay_protection() {
        use base64::prelude::*;
        use styks_blocky_parser::blocky_claims::BlockyClaims;

        let (env, _feed, mut supplier, config, blocky_output) = setup();

        // Setup similar to test_report_prices_with_cached_signer
        let enclave_att = &blocky_output
            .enclave_attested_application_public_key
            .enclave_attestation;
        let (platform, code, _pubkey) =
            styks_blocky_parser::nitro::extract_measurement_from_attestation(enclave_att)
                .expect("Failed to extract measurement");

        let mut new_config = config.clone();
        new_config.allowed_measurements = vec![MeasurementRule {
            platform: platform.clone(),
            code: code.clone(),
        }];
        new_config.timestamp_tolerance = 20 * 60;
        supplier.set_config(new_config);

        let pubkey = Bytes::from(blocky_output.public_key_bytes());
        let signer_id = supplier.register_signer_manual(pubkey, platform, code);

        // Get timestamp from TA claims
        let ta = blocky_output.ta();
        let claims = BlockyClaims::decode_fn_call_claims(ta.data()).expect("decode claims");
        let output = claims.output().expect("get output");
        let timestamp = output.timestamp;
        env.advance_block_time(timestamp * 1000);

        let ta_b64 = &blocky_output.transitive_attested_function_call.transitive_attestation;
        let ta_bytes = BASE64_STANDARD.decode(ta_b64).expect("Failed to decode TA");

        // First submission should succeed
        supplier.report_prices(Bytes::from(ta_bytes.clone()), Some(signer_id.clone()), None);

        // Second submission with same timestamp should fail (replay)
        let result = supplier.try_report_prices(Bytes::from(ta_bytes), Some(signer_id), None);
        assert!(result.is_err());
    }
}
