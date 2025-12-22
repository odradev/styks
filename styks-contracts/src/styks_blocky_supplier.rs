use odra::{casper_types::bytesrepr::Bytes, prelude::*, ContractRef};
use odra_modules::access::{AccessControl, Role, DEFAULT_ADMIN_ROLE};
use sha3::{Digest, Keccak256};

// Mapping and Var are re-exported from prelude
use styks_blocky_parser::{
    blocky_claims::{BlockyClaims, BlockyClaimsError},
    enclave_attestation::{parse_enclave_attestation_with_pubkey, EnclaveAttestationError},
    transitive_attestation::{decode_transitive_attestation, TAError},
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

    // Measurement anchored model errors.
    ContractPaused = 46400,
    SignerNotFound = 46401,
    SignerRevoked = 46402,
    SignerExpired = 46403,
    MeasurementNotAllowed = 46404,
    InvalidEnclaveAttestation = 46405,
    ReplayAttack = 46406,
    MissingSignerInfo = 46407,
    SignatureTooShort = 46408,
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

impl From<TAError> for StyksBlockySupplierError {
    fn from(error: TAError) -> Self {
        use TAError::*;
        match error {
            DecodeFailed => StyksBlockySupplierError::TADataDecoding,
            InvalidLength => StyksBlockySupplierError::TADataInvalidLength,
            BytesConversionError => StyksBlockySupplierError::BytesConversionError,
            SignatureTooShort => StyksBlockySupplierError::SignatureTooShort,
        }
    }
}

impl From<EnclaveAttestationError> for StyksBlockySupplierError {
    fn from(_error: EnclaveAttestationError) -> Self {
        StyksBlockySupplierError::InvalidEnclaveAttestation
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
            // start with 3, so it doesn't overlap with PriceFeed.
            StyksBlockySupplierRole::ConfigManager => [3u8; 32],
            StyksBlockySupplierRole::Guardian => [4u8; 32],
        }
    }
}

// --- Data Types ---

/// A measurement rule that defines an allowlisted enclave measurement.
#[odra::odra_type]
pub struct MeasurementRule {
    /// The attestation platform (e.g., "nitro").
    pub platform: String,
    /// The measurement code (PCR values, e.g., "pcr0.pcr1.pcr2").
    pub code: String,
}

/// Configuration for the StyksBlockySupplier contract.
#[odra::odra_type]
pub struct StyksBlockySupplierConfig {
    /// Expected SHA3-512 hash of the guest WASM program.
    pub wasm_hash: String,
    /// Expected function name (e.g., "priceFunc").
    pub expected_function: String,
    /// Allowlisted enclave measurements.
    pub allowed_measurements: Vec<MeasurementRule>,
    /// Mapping from CoinGecko identifier to PriceFeedId.
    pub coingecko_feed_ids: Vec<(String, PriceFeedId)>,
    /// Address of the StyksPriceFeed contract.
    pub price_feed_address: Address,
    /// Maximum age of a price report in seconds.
    pub timestamp_tolerance: u64,
    /// Time-to-live for cached signers in seconds (0 = no expiry).
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

    /// Checks if a measurement is in the allowlist.
    pub fn is_measurement_allowed(&self, platform: &str, code: &str) -> bool {
        self.allowed_measurements
            .iter()
            .any(|m| m.platform == platform && m.code == code)
    }
}

/// A cached signer entry.
#[odra::odra_type]
pub struct CachedSigner {
    /// The public key in SEC1 format.
    pub pubkey_sec1: Bytes,
    /// The attestation platform.
    pub measurement_platform: String,
    /// The measurement code (PCRs).
    pub measurement_code: String,
    /// Timestamp when the signer was registered.
    pub registered_at: u64,
    /// Timestamp of the last price report from this signer.
    pub last_seen: u64,
    /// Whether the signer has been revoked.
    pub revoked: bool,
}

// --- StyksBlockySupplier Contract ---

#[odra::module]
pub struct StyksBlockySupplier {
    access_control: SubModule<AccessControl>,
    config: Var<StyksBlockySupplierConfig>,
    /// Cached signers, keyed by signer_id (keccak256 of pubkey).
    cached_signers: Mapping<Bytes, CachedSigner>,
    /// Last accepted timestamp per price feed (for replay protection).
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
        self.assert_config_manager(&self.env().caller());
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

    /// Pauses the contract. Only Guardian or Admin can call this.
    pub fn pause(&mut self) {
        self.assert_guardian_or_admin(&self.env().caller());
        self.paused.set(true);
    }

    /// Unpauses the contract. Only Guardian or Admin can call this.
    pub fn unpause(&mut self) {
        self.assert_guardian_or_admin(&self.env().caller());
        self.paused.set(false);
    }

    /// Returns whether the contract is paused.
    pub fn is_paused(&self) -> bool {
        self.paused.get_or_default()
    }

    // --- Signer Management ---

    /// Revokes a signer. Only Guardian or Admin can call this.
    pub fn revoke_signer(&mut self, signer_id: Bytes) {
        self.assert_guardian_or_admin(&self.env().caller());

        let mut signer = self.cached_signers
            .get(&signer_id)
            .unwrap_or_revert_with(&self.env(), StyksBlockySupplierError::SignerNotFound);

        signer.revoked = true;
        self.cached_signers.set(&signer_id, signer);
    }

    /// Registers a new signer from enclave attestation claims.
    ///
    /// # Arguments
    /// * `enclave_attestation_claims` - JSON bytes of the attestation claims
    /// * `public_key` - Raw SEC1 public key bytes (decoded from base64 by CLI)
    ///
    /// # Returns
    /// The signer_id (keccak256 hash of the public key)
    pub fn register_signer(
        &mut self,
        enclave_attestation_claims: Bytes,
        public_key: Bytes,
    ) -> Bytes {
        self.assert_not_paused();
        let config = self.get_config();
        let current_time = self.env().get_block_time_secs();

        // Parse the attestation claims
        let attestation = match parse_enclave_attestation_with_pubkey(
            &enclave_attestation_claims,
            public_key.to_vec(),
        ) {
            Ok(a) => a,
            Err(_) => self.env().revert(StyksBlockySupplierError::InvalidEnclaveAttestation),
        };

        // Check if the measurement is allowlisted
        if !config.is_measurement_allowed(&attestation.platform, &attestation.measurement_code) {
            self.env().revert(StyksBlockySupplierError::MeasurementNotAllowed);
        }

        // Compute signer_id = keccak256(pubkey)
        let signer_id = Self::compute_signer_id(&attestation.public_key);

        // Create cached signer entry
        let cached = CachedSigner {
            pubkey_sec1: Bytes::from(attestation.public_key),
            measurement_platform: attestation.platform,
            measurement_code: attestation.measurement_code,
            registered_at: current_time,
            last_seen: current_time,
            revoked: false,
        };

        // Store the signer
        self.cached_signers.set(&signer_id, cached);

        signer_id
    }

    /// Gets a cached signer by ID.
    pub fn get_signer(&self, signer_id: &Bytes) -> Option<CachedSigner> {
        self.cached_signers.get(signer_id)
    }

    // --- Price Reporting ---

    /// Reports prices using the new measurement-anchored model.
    ///
    /// # Arguments
    /// * `transitive_attestation` - Raw TA bytes (ABI-encoded)
    /// * `signer_id` - Optional signer ID for fast path
    /// * `enclave_attestation_claims` - Optional attestation claims for slow path
    /// * `public_key` - Optional public key bytes (required if enclave_attestation_claims is provided)
    pub fn report_prices(
        &mut self,
        transitive_attestation: Bytes,
        signer_id: Option<Bytes>,
        enclave_attestation_claims: Option<Bytes>,
        public_key: Option<Bytes>,
    ) {
        self.assert_not_paused();
        let config = self.get_config();
        let current_time = self.env().get_block_time_secs();

        // Decode the transitive attestation
        let (data, signature) = match decode_transitive_attestation(&transitive_attestation) {
            Ok(result) => result,
            Err(e) => self.env().revert(StyksBlockySupplierError::from(e)),
        };

        // Resolve the public key for verification
        let (pubkey_bytes, resolved_signer_id) = if let Some(sid) = signer_id {
            // Fast path: use cached signer
            let signer = self.get_valid_signer(&sid, &config, current_time);
            (signer.pubkey_sec1.to_vec(), sid)
        } else if let Some(ea_claims) = enclave_attestation_claims {
            // Slow path: register new signer inline
            let pk = match public_key {
                Some(p) => p,
                None => self.env().revert(StyksBlockySupplierError::MissingSignerInfo),
            };
            let new_signer_id = self.register_signer(ea_claims, pk.clone());
            (pk.to_vec(), new_signer_id)
        } else {
            self.env().revert(StyksBlockySupplierError::MissingSignerInfo);
        };

        // Verify the signature
        self.assert_valid_signature(&pubkey_bytes, &signature, &data);

        // Decode and verify claims
        let claims = match BlockyClaims::decode_fn_call_claims(&data) {
            Ok(c) => c,
            Err(e) => self.env().revert(StyksBlockySupplierError::from(e)),
        };

        // Verify WASM hash
        if claims.hash_of_code() != config.wasm_hash {
            self.env().revert(StyksBlockySupplierError::BadWasmHash);
        }

        // Verify function name
        if claims.function() != config.expected_function {
            self.env().revert(StyksBlockySupplierError::BadFunctionName);
        }

        // Extract output
        let output = match claims.output() {
            Ok(o) => o,
            Err(e) => self.env().revert(StyksBlockySupplierError::from(e)),
        };

        // Verify timestamp tolerance
        self.assert_timestamp_in_range(output.timestamp, config.timestamp_tolerance);

        // Get price feed ID
        let price_feed_id = match config.price_feed_id(&output.identifier()) {
            Some(id) => id,
            None => self.env().revert(StyksBlockySupplierError::PriceFeedIdNotFound),
        };

        // Replay protection: ensure timestamp is newer than last accepted
        let last_ts = self.last_accepted_timestamp.get(&price_feed_id).unwrap_or(0);
        if output.timestamp <= last_ts {
            self.env().revert(StyksBlockySupplierError::ReplayAttack);
        }

        // Update last accepted timestamp
        self.last_accepted_timestamp.set(&price_feed_id, output.timestamp);

        // Update signer's last_seen
        if let Some(mut signer) = self.cached_signers.get(&resolved_signer_id) {
            signer.last_seen = current_time;
            self.cached_signers.set(&resolved_signer_id, signer);
        }

        // Forward price to feed
        let mut feed = StyksPriceFeedContractRef::new(self.env(), config.price_feed_address);
        let price = Price::from(output.price);
        feed.add_to_feed(vec![(price_feed_id, price)]);
    }
}

// --- Private Implementation ---

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
            self.env().revert(StyksBlockySupplierError::NotGuardianRole);
        }
    }

    fn assert_not_paused(&self) {
        if self.is_paused() {
            self.env().revert(StyksBlockySupplierError::ContractPaused);
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
        if reported < current_time.saturating_sub(tolerance) || reported > current_time + tolerance {
            self.env().revert(StyksBlockySupplierError::TimestampOutOfRange);
        }
    }

    fn get_valid_signer(
        &self,
        signer_id: &Bytes,
        config: &StyksBlockySupplierConfig,
        current_time: u64,
    ) -> CachedSigner {
        let signer = self.cached_signers
            .get(signer_id)
            .unwrap_or_revert_with(&self.env(), StyksBlockySupplierError::SignerNotFound);

        // Check if revoked
        if signer.revoked {
            self.env().revert(StyksBlockySupplierError::SignerRevoked);
        }

        // Check if expired (if TTL is set)
        if config.signer_ttl_secs > 0 {
            let age = current_time.saturating_sub(signer.registered_at);
            if age > config.signer_ttl_secs {
                self.env().revert(StyksBlockySupplierError::SignerExpired);
            }
        }

        signer
    }

    fn compute_signer_id(pubkey: &[u8]) -> Bytes {
        let mut hasher = Keccak256::new();
        hasher.update(pubkey);
        let hash = hasher.finalize();
        Bytes::from(hash.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use odra::host::{Deployer, HostEnv, NoArgs};
    use styks_blocky_parser::blocky_output::BlockyOutput;

    use crate::styks_price_feed::{StyksPriceFeed, StyksPriceFeedConfig, StyksPriceFeedHostRef, StyksPriceFeedRole};

    use super::*;

    fn setup() -> (HostEnv, StyksPriceFeedHostRef, StyksBlockySupplierHostRef, StyksBlockySupplierConfig, BlockyOutput) {
        let env = odra_test::env();
        let admin = env.get_account(0);

        // Load BlockyOutput from file.
        let blocky_output = BlockyOutput::try_from_file("../resources/test/2_out.json")
            .expect("Failed to load BlockyOutput");

        // Load guest wasm bytes.
        let wasm_bytes = include_bytes!("../../resources/test/1_guest.wasm");
        let wasm_hash = styks_blocky_parser::wasm_hash(wasm_bytes);

        // Get the measurement from the blocky output
        let measurement = &blocky_output.enclave_attested_application_public_key.claims.enclave_measurement;

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
                platform: measurement.platform.clone(),
                code: measurement.code.clone(),
            }],
            coingecko_feed_ids: vec![
                (String::from("Gate_CSPR_USD"), String::from("CSPRUSD"))
            ],
            price_feed_address: feed.address(),
            timestamp_tolerance: 60, // 60 sec tolerance for tests
            signer_ttl_secs: 86400, // 24 hours
        };
        supplier.grant_role(&StyksBlockySupplierRole::ConfigManager.role_id(), &admin);
        supplier.set_config(supplier_config.clone());

        // Allow StyksBlockySupplier to add prices to StyksPriceFeed.
        let role = StyksPriceFeedRole::PriceSupplier.role_id();
        feed.grant_role(&role, &supplier.address());

        (env, feed, supplier, supplier_config, blocky_output)
    }

    #[test]
    fn test_register_signer_and_report_prices() {
        let (env, feed, mut supplier, supplier_config, blocky_output) = setup();
        let id = supplier_config.coingecko_feed_ids[0].1.clone();

        // Advance time to match the test data timestamp
        let timestamp = 1765796826u64;
        env.advance_block_time(timestamp * 1000);

        // Price should be empty initially
        assert_eq!(feed.get_twap_price(&id), None);

        // Get attestation claims and public key
        let claims_json = blocky_output.enclave_attestation_claims_json();
        let public_key = blocky_output.public_key_bytes();

        // Register the signer
        let signer_id = supplier.register_signer(
            Bytes::from(claims_json),
            Bytes::from(public_key),
        );

        // Verify signer was cached
        let signer = supplier.get_signer(&signer_id);
        assert!(signer.is_some());
        let signer = signer.unwrap();
        assert!(!signer.revoked);
        assert_eq!(signer.measurement_platform, "nitro");

        // Get TA bytes
        let ta_bytes = blocky_output.transitive_attestation_bytes();

        // Report prices using the cached signer (fast path)
        supplier.report_prices(
            Bytes::from(ta_bytes),
            Some(signer_id),
            None,
            None,
        );

        // Check the reported price
        let price = feed.get_twap_price(&id);
        assert_eq!(price, Some(516));
    }

    #[test]
    fn test_pause_unpause() {
        let (env, _feed, mut supplier, _supplier_config, blocky_output) = setup();
        let admin = env.get_account(0);

        // Grant guardian role to admin for testing
        supplier.grant_role(&StyksBlockySupplierRole::Guardian.role_id(), &admin);

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
    fn test_revoke_signer() {
        let (env, _feed, mut supplier, _supplier_config, blocky_output) = setup();
        let admin = env.get_account(0);

        // Grant guardian role
        supplier.grant_role(&StyksBlockySupplierRole::Guardian.role_id(), &admin);

        // Register a signer
        let claims_json = blocky_output.enclave_attestation_claims_json();
        let public_key = blocky_output.public_key_bytes();
        let signer_id = supplier.register_signer(
            Bytes::from(claims_json),
            Bytes::from(public_key),
        );

        // Verify not revoked
        let signer = supplier.get_signer(&signer_id).unwrap();
        assert!(!signer.revoked);

        // Revoke
        supplier.revoke_signer(signer_id.clone());

        // Verify revoked
        let signer = supplier.get_signer(&signer_id).unwrap();
        assert!(signer.revoked);
    }

    #[test]
    fn test_measurement_not_allowed() {
        let (env, _feed, mut supplier, _supplier_config, blocky_output) = setup();
        let admin = env.get_account(0);

        // Update config with empty allowlist
        let mut config = supplier.get_config();
        config.allowed_measurements = vec![];
        supplier.set_config(config);

        // Try to register signer - should fail
        let claims_json = blocky_output.enclave_attestation_claims_json();
        let public_key = blocky_output.public_key_bytes();

        let result = supplier.try_register_signer(
            Bytes::from(claims_json),
            Bytes::from(public_key),
        );
        assert!(result.is_err());
    }
}
