use odra::{casper_types::bytesrepr::Bytes, prelude::*};
use odra_modules::access::{AccessControl, Role, DEFAULT_ADMIN_ROLE};
use styks_blocky_parser::{blocky_claims::{BlockyClaims, BlockyClaimsError}, verify::VerificationError};
use styks_core::{Price, PriceFeedId};

// --- Errors ---

#[odra::odra_error]
pub enum StyksBlockySupplerError {
    // Config errors.
    ConfigNotSet = 46000,

     // Role errors.
    NotAdminRole = 46100,
    NotConfigManagerRole = 46101,

    // Verification errors.
    InvalidPublicKey = 46200,
    InvalidSignature = 46201,
    HashingError = 46202,
    BadSignature = 46203,
    BadWasmHash = 46204,

    // Claims errors.
    TADataDecoding = 46300,
    TADataInvalidLength = 46301,
    BytesConversionError = 46302,
    OutputJsonDecoding = 46303,
    OutputHasNoSuccessStatus = 46304,

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
    ConfigManager
}

impl StyksBlockySupplerRole {
    pub fn role_id(&self) -> Role {
        match self {
            StyksBlockySupplerRole::Admin => DEFAULT_ADMIN_ROLE,
            // start with 3, so it doesn't overlap with PriceFeed.
            StyksBlockySupplerRole::ConfigManager => [3u8; 32],
        }
    }
}

// --- Configuration ---

#[odra::odra_type]
pub struct StyksBlockySupplerConfig {
    pub wasm_hash: String,
    pub public_key: Bytes,
    pub coingecko_feed_ids: Vec<(String, PriceFeedId)>, // (coingecko_id, price_feed_id)
}

impl StyksBlockySupplerConfig {
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }
}

// --- StyksBlockySupplier Contract ---

#[odra::module]
pub struct StyksBlockySupplier {
    access_control: SubModule<AccessControl>,
    config: Var<StyksBlockySupplerConfig>,
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

        // Validate the config.
        // config.validate().unwrap_or_revert(&self.env());

        // Update the config.
        self.config.set(config);
    }

    pub fn get_config(&self) -> StyksBlockySupplerConfig {
        self.config
            .get()
            .unwrap_or_revert_with(&self.env(), StyksBlockySupplerError::ConfigNotSet)
    }

    /// Verifies the signature against the data.
    pub fn report_signed_prices(
        &mut self,
        signature: Bytes,
        data: Bytes,
    ) {
        let config = self.get_config();
        let public_key = config.public_key();

        // Verify the signature.
        self.assert_valid_signature(&public_key, &signature, &data);

        // Decode the data.
        let claims = match BlockyClaims::decode_fn_call_claims(&data) {
            Ok(claims) => claims,
            Err(error) => {
                self.env().revert(StyksBlockySupplerError::from(error));
            }
        };
        
        // Verify the claims.
        if claims.hash_of_code() != config.wasm_hash {
            self.env().revert(StyksBlockySupplerError::BadWasmHash);
        }

        let output = match claims.output() {
            Ok(output) => output,
            Err(error) => {
                self.env().revert(StyksBlockySupplerError::from(error));
            }
        };

        let price = output.price;
        let price: Price = (price * 100_000.0) as u64;

        if price == 0 {
            self.env().revert(StyksBlockySupplerError::OutputHasNoSuccessStatus);
        }
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
            };
            self.env().revert(error);
        }
    }

    fn assert_config_manager(&self, address: &Address) {
        self.assert_role(address, StyksBlockySupplerRole::ConfigManager);
    }

    fn assert_valid_signature(
        &self,
        public_key: &[u8],
        signature: &[u8],
        data: &[u8],
    ) {
        let result = styks_blocky_parser::verify::verify_signature(
            public_key,
            signature,
            data,
        );
        if let Err(error) = result {
            self.env().revert(StyksBlockySupplerError::from(error));
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
            heartbeat_tolerance: 10,
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
            public_key: Bytes::from(blocky_output.public_key_bytes()),
            coingecko_feed_ids: vec![
                (String::from("casper_network"), String::from("CSPRUSD"))
            ],
        };
        supplier.grant_role(&StyksBlockySupplerRole::ConfigManager.role_id(), &admin);
        supplier.set_config(supplier_config.clone());

        // Allow StyksBlockySupplier to add prices to StyksPriceFeed.
        let role = StyksPriceFeedRole::ConfigManager.role_id();
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
        env.advance_block_time(100 * 1000);
        assert_eq!(100, env.block_time_secs());
        
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
        
    }
}