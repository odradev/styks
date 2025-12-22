use odra::{casper_types::bytesrepr::Bytes, prelude::*, ContractRef};
use odra_modules::access::{AccessControl, Role, DEFAULT_ADMIN_ROLE};
use styks_blocky_parser::{blocky_claims::{BlockyClaims, BlockyClaimsError}, verify::VerificationError};
use styks_core::{Price, PriceFeedId};

use crate::styks_price_feed::StyksPriceFeedContractRef;

// --- Errors ---

#[odra::odra_error]
pub enum StyksBlockySupplerError {
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

    // Claims errors.
    TADataDecoding = 46300,
    TADataInvalidLength = 46301,
    BytesConversionError = 46302,
    OutputJsonDecoding = 46303,
    OutputHasNoSuccessStatus = 46304,

    // Key ring errors.
    DuplicateSignerKey = 46400,
    SignerKeyNotFound = 46401,
    BadFunctionName = 46402,
    TimestampNotMonotonic = 46403,
    ContractPaused = 46404,
    NoSignerKeys = 46405,
}

impl From<VerificationError> for StyksBlockySupplerError {
    fn from(error: VerificationError) -> Self {
        use VerificationError::*;
        match error {
            InvalidPublicKey => StyksBlockySupplerError::InvalidPublicKey,
            InvalidSignature => StyksBlockySupplerError::InvalidSignature,
            HashingError => StyksBlockySupplerError::HashingError,
            BadSignature => StyksBlockySupplerError::BadSignature,
        }
    }
}

impl From<BlockyClaimsError> for StyksBlockySupplerError {
    fn from(error: BlockyClaimsError) -> Self {
        use BlockyClaimsError::*;
        match error {
            TADataDecoding => StyksBlockySupplerError::TADataDecoding,
            TADataInvalidLength => StyksBlockySupplerError::TADataInvalidLength,
            BytesConversionError => StyksBlockySupplerError::BytesConversionError,
            OutputJsonDecoding => StyksBlockySupplerError::OutputJsonDecoding,
            OutputHasNoSuccessStatus => StyksBlockySupplerError::OutputHasNoSuccessStatus,
        }
    }
}

// --- Access Control Roles ---

#[derive(Debug)]
pub enum StyksBlockySupplerRole {
    Admin,
    ConfigManager,
    Guardian,
}

impl StyksBlockySupplerRole {
    pub fn role_id(&self) -> Role {
        match self {
            StyksBlockySupplerRole::Admin => DEFAULT_ADMIN_ROLE,
            // start with 3, so it doesn't overlap with PriceFeed.
            StyksBlockySupplerRole::ConfigManager => [3u8; 32],
            StyksBlockySupplerRole::Guardian => [4u8; 32],
        }
    }
}

// --- Configuration ---

#[odra::odra_type]
pub struct StyksBlockySupplerConfig {
    pub wasm_hash: String,
    pub coingecko_feed_ids: Vec<(String, PriceFeedId)>, // (coingecko_id, price_feed_id)
    pub price_feed_address: Address,
    pub timestamp_tolerance: u64,
}

impl StyksBlockySupplerConfig {
    pub fn price_feed_id(&self, coingecko_id: &str) -> Option<PriceFeedId> {
        self.coingecko_feed_ids
            .iter()
            .find(|(id, _)| id == coingecko_id)
            .map(|(_, feed_id)| feed_id.clone())
    }
}

// --- Signer Key Record ---

#[odra::odra_type]
pub struct SignerKeyRecord {
    pub public_key: Bytes,
    pub not_before: u64,
    pub not_after: u64,
    pub revoked: bool,
}

impl SignerKeyRecord {
    pub fn is_active(&self, now: u64) -> bool {
        if self.revoked {
            return false;
        }
        if self.not_before != 0 && now < self.not_before {
            return false;
        }
        if self.not_after != 0 && now > self.not_after {
            return false;
        }
        true
    }
}

// --- Events ---

#[odra::event]
pub struct SignerKeyAdded {
    pub by: Address,
    pub public_key: Bytes,
    pub not_before: u64,
    pub not_after: u64,
}

#[odra::event]
pub struct SignerKeyRetired {
    pub by: Address,
    pub public_key: Bytes,
    pub not_after: u64,
}

#[odra::event]
pub struct SignerKeyRevoked {
    pub by: Address,
    pub public_key: Bytes,
}

#[odra::event]
pub struct Paused {
    pub account: Address,
}

#[odra::event]
pub struct Unpaused {
    pub account: Address,
}

// --- StyksBlockySupplier Contract ---

#[odra::module(
    events = [SignerKeyAdded, SignerKeyRetired, SignerKeyRevoked, Paused, Unpaused],
    errors = StyksBlockySupplerError
)]
pub struct StyksBlockySupplier {
    access_control: SubModule<AccessControl>,
    config: Var<StyksBlockySupplerConfig>,
    // Key ring storage
    signer_keys: Var<Vec<SignerKeyRecord>>,
    is_paused: Var<bool>,
    last_seen_timestamp: Mapping<PriceFeedId, u64>,
    expected_function: Var<String>,
}

#[odra::module]
impl StyksBlockySupplier {
    pub fn init(&mut self) {
        // Grant the admin role to the contract deployer.
        let deployer = self.env().caller();
        let admin_role = StyksBlockySupplerRole::Admin.role_id();
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

    pub fn set_config(&mut self, config: StyksBlockySupplerConfig) {
        // Make sure only ConfigManager can set the config.
        self.assert_config_manager(&self.env().caller());

        // Update the config.
        self.config.set(config);
    }

    pub fn get_config(&self) -> StyksBlockySupplerConfig {
        self.config
            .get()
            .unwrap_or_revert_with(&self.env(), StyksBlockySupplerError::ConfigNotSet)
    }

    pub fn get_config_or_none(&self) -> Option<StyksBlockySupplerConfig> {
        self.config.get()
    }

    /// Verifies the signature against the data and reports prices to the feed.
    pub fn report_signed_prices(
        &mut self,
        signature: Bytes,
        data: Bytes,
    ) {
        // 1. Pause gate first
        self.require_not_paused();

        let config = self.get_config();
        let now = self.env().get_block_time_secs();

        // 2. Signature verification with key ring
        let keys = self.signer_keys.get_or_default();
        if keys.is_empty() {
            self.env().revert(StyksBlockySupplerError::NoSignerKeys);
        }
        self.assert_valid_signature_any(&keys, &signature, &data, now);

        // 3. Decode the data
        let claims = match BlockyClaims::decode_fn_call_claims(&data) {
            Ok(claims) => claims,
            Err(error) => {
                self.env().revert(StyksBlockySupplerError::from(error));
            }
        };

        // 4. Verify WASM hash
        if claims.hash_of_code() != config.wasm_hash {
            self.env().revert(StyksBlockySupplerError::BadWasmHash);
        }

        // 5. Enforce function name (if configured)
        let expected_fn = self.expected_function.get_or_default();
        if !expected_fn.is_empty() && claims.function() != expected_fn {
            self.env().revert(StyksBlockySupplerError::BadFunctionName);
        }

        // 6. Extract the output
        let output = match claims.output() {
            Ok(output) => output,
            Err(error) => {
                self.env().revert(StyksBlockySupplerError::from(error));
            }
        };

        // 7. Verify timestamp freshness
        self.assert_timestamp_in_range(output.timestamp, config.timestamp_tolerance);

        // 8. Load price feed ID
        let price_feed_id = match config.price_feed_id(&output.identifier()) {
            Some(id) => PriceFeedId::from(id),
            None => self.env().revert(StyksBlockySupplerError::PriceFeedIdNotFound)
        };

        // 9. Monotonic timestamp anti-replay
        let last = self.last_seen_timestamp.get(&price_feed_id).unwrap_or_default();
        if output.timestamp <= last {
            self.env().revert(StyksBlockySupplerError::TimestampNotMonotonic);
        }

        // 10. Forward to price feed
        let mut feed = StyksPriceFeedContractRef::new(
            self.env(),
            config.price_feed_address,
        );
        let price = Price::from(output.price);
        feed.add_to_feed(vec![(price_feed_id.clone(), price)]);

        // 11. Update last seen timestamp
        self.last_seen_timestamp.set(&price_feed_id, output.timestamp);
    }

    // --- Key Ring Management ---

    pub fn get_signer_keys(&self) -> Vec<SignerKeyRecord> {
        self.signer_keys.get_or_default()
    }

    pub fn add_signer_key(&mut self, public_key: Bytes, not_before: u64, not_after: u64) {
        self.assert_config_manager(&self.env().caller());

        // Validate public key format
        if let Err(e) = styks_blocky_parser::verify::validate_public_key(&public_key) {
            self.env().revert(StyksBlockySupplerError::from(e));
        }

        // Check for duplicates
        let mut keys = self.signer_keys.get_or_default();
        for key in &keys {
            if key.public_key == public_key {
                self.env().revert(StyksBlockySupplerError::DuplicateSignerKey);
            }
        }

        let record = SignerKeyRecord {
            public_key: public_key.clone(),
            not_before,
            not_after,
            revoked: false,
        };
        keys.push(record);
        self.signer_keys.set(keys);

        self.env().emit_event(SignerKeyAdded {
            by: self.env().caller(),
            public_key,
            not_before,
            not_after,
        });
    }

    pub fn retire_signer_key(&mut self, public_key: Bytes, not_after: u64) {
        self.assert_config_manager(&self.env().caller());

        let mut keys = self.signer_keys.get_or_default();
        let mut found = false;
        for key in &mut keys {
            if key.public_key == public_key {
                key.not_after = not_after;
                found = true;
                break;
            }
        }
        if !found {
            self.env().revert(StyksBlockySupplerError::SignerKeyNotFound);
        }
        self.signer_keys.set(keys);

        self.env().emit_event(SignerKeyRetired {
            by: self.env().caller(),
            public_key,
            not_after,
        });
    }

    pub fn revoke_signer_key(&mut self, public_key: Bytes) {
        // Guardian OR ConfigManager can revoke
        let caller = self.env().caller();
        let is_guardian = self.has_role(&StyksBlockySupplerRole::Guardian.role_id(), &caller);
        let is_config_manager = self.has_role(&StyksBlockySupplerRole::ConfigManager.role_id(), &caller);
        if !is_guardian && !is_config_manager {
            self.env().revert(StyksBlockySupplerError::NotGuardianRole);
        }

        let mut keys = self.signer_keys.get_or_default();
        let mut found = false;
        for key in &mut keys {
            if key.public_key == public_key {
                key.revoked = true;
                found = true;
                break;
            }
        }
        if !found {
            self.env().revert(StyksBlockySupplerError::SignerKeyNotFound);
        }
        self.signer_keys.set(keys);

        self.env().emit_event(SignerKeyRevoked {
            by: self.env().caller(),
            public_key,
        });
    }

    pub fn set_expected_function(&mut self, name: String) {
        self.assert_config_manager(&self.env().caller());
        self.expected_function.set(name);
    }

    pub fn get_expected_function(&self) -> String {
        self.expected_function.get_or_default()
    }

    // --- Pause Control ---

    pub fn pause(&mut self) {
        self.assert_guardian_or_admin(&self.env().caller());
        self.is_paused.set(true);
        self.env().emit_event(Paused {
            account: self.env().caller(),
        });
    }

    pub fn unpause(&mut self) {
        self.assert_guardian_or_admin(&self.env().caller());
        self.is_paused.set(false);
        self.env().emit_event(Unpaused {
            account: self.env().caller(),
        });
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.get_or_default()
    }
}

impl StyksBlockySupplier {
    fn assert_role(&self, address: &Address, role: StyksBlockySupplerRole) {
        if !self.has_role(&role.role_id(), address) {
            use StyksBlockySupplerError::*;
            use StyksBlockySupplerRole::*;
            let error = match role {
                Admin => NotAdminRole,
                ConfigManager => NotConfigManagerRole,
                Guardian => NotGuardianRole,
            };
            self.env().revert(error);
        }
    }

    fn assert_config_manager(&self, address: &Address) {
        self.assert_role(address, StyksBlockySupplerRole::ConfigManager);
    }

    fn assert_guardian_or_admin(&self, address: &Address) {
        let is_guardian = self.has_role(&StyksBlockySupplerRole::Guardian.role_id(), address);
        let is_admin = self.has_role(&StyksBlockySupplerRole::Admin.role_id(), address);
        if !is_guardian && !is_admin {
            self.env().revert(StyksBlockySupplerError::NotGuardianRole);
        }
    }

    fn require_not_paused(&self) {
        if self.is_paused.get_or_default() {
            self.env().revert(StyksBlockySupplerError::ContractPaused);
        }
    }

    fn assert_valid_signature_any(
        &self,
        keys: &[SignerKeyRecord],
        signature: &[u8],
        data: &[u8],
        now: u64,
    ) {
        for key in keys {
            if !key.is_active(now) {
                continue;
            }
            let result = styks_blocky_parser::verify::verify_signature(
                &key.public_key,
                signature,
                data,
            );
            if result.is_ok() {
                return; // Found valid signature
            }
        }
        self.env().revert(StyksBlockySupplerError::BadSignature);
    }

    fn assert_timestamp_in_range(&self, reported: u64, tolerance: u64) {
        let current_time = self.env().get_block_time_secs();
        if reported < current_time.saturating_sub(tolerance) || reported > current_time + tolerance {
            self.env().revert(StyksBlockySupplerError::TimestampOutOfRange);
        }
    }
}

#[cfg(test)]
mod tests {
    use odra::{host::{Deployer, HostEnv, NoArgs}};
    use styks_blocky_parser::blocky_output::BlockyOutput;

    use crate::styks_price_feed::{StyksPriceFeed, StyksPriceFeedConfig, StyksPriceFeedHostRef, StyksPriceFeedRole};

    use super::*;

    fn setup() -> (HostEnv, StyksPriceFeedHostRef, StyksBlockySupplierHostRef, StyksBlockySupplerConfig, BlockyOutput) {
        
        let env = odra_test::env();
        let admin = env.get_account(0);
        
        // Load BlockyOutput from file.
        let blocky_output = BlockyOutput::try_from_file("../resources/test/1_out.json")
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
        let supplier_config = StyksBlockySupplerConfig {
            wasm_hash,
            coingecko_feed_ids: vec![
                (String::from("Gate_CSPR_USD"), String::from("CSPRUSD"))
            ],
            price_feed_address: feed.address(),
            timestamp_tolerance: 1, // 1 sec tolerance
        };
        supplier.grant_role(&StyksBlockySupplerRole::ConfigManager.role_id(), &admin);
        supplier.set_config(supplier_config.clone());

        // Add the signer key to the key ring (required for signature verification).
        let public_key = Bytes::from(blocky_output.public_key_bytes());
        supplier.add_signer_key(public_key, 0, 0);

        // Allow StyksBlockySupplier to add prices to StyksPriceFeed.
        let role = StyksPriceFeedRole::PriceSupplier.role_id();
        feed.grant_role(&role, &supplier.address());

        (env, feed, supplier, supplier_config, blocky_output)
    }

    #[test]
    fn test_styks_blocky_supplier() {
        let (env, feed, mut supplier, supplier_config, blocky_output) = setup();
        let id = supplier_config.coingecko_feed_ids[0].1.clone();

        // Check initial config.
        assert_eq!(supplier.get_config(), supplier_config);

        // Assuming the test starts at block time 1000.
        let timestamp = 1755463157;
        env.advance_block_time(timestamp * 1000);
        assert_eq!(timestamp, env.block_time_secs());
        
        // Price should be empty initially.
        assert_eq!(feed.get_twap_price(&id), None);

        // Report prices using the supplier.
        let ta = blocky_output.ta();
        let signature = ta.signature_bytes();
        let data = ta.data();

        supplier.report_signed_prices(
            Bytes::from(signature),
            Bytes::from(data),
        );

        // Check the reported price.
        let price = feed.get_twap_price(&id);
        assert_eq!(price, Some(1056));
    }
}
