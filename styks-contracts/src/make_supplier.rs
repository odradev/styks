use odra::{
    casper_types::{
        bytesrepr::{Bytes, FromBytes},
        PublicKey,
    },
    prelude::*,
    ContractRef,
};
use odra_modules::access::{AccessControl, Role};
use sha2::{Digest, Sha256};
use styks_core::PriceFeedId;

use crate::{
    styks_price_feed::StyksPriceFeedContractRef, supplier_error::StyksSupplierError,
    supplier_role::SupplierRole,
};

// --- Configuration ---
#[odra::odra_type]
pub struct MakeSupplierConfig {
    pub public_key: PublicKey,
    pub feed_ids: Vec<(String, PriceFeedId)>, // (make_id, price_feed_id)
    pub price_feed_address: Address,
    pub timestamp_tolerance: u64,
}

impl MakeSupplierConfig {
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn price_feed_id(&self, make_id: &str) -> Option<&PriceFeedId> {
        self.feed_ids
            .iter()
            .find(|(id, _)| id == make_id)
            .map(|(_, feed_id)| feed_id)
    }
}

// --- StyksBlockySupplier Contract ---

#[odra::module]
pub struct StyksMakeSupplier {
    access_control: SubModule<AccessControl>,
    config: Var<MakeSupplierConfig>,
}

#[odra::module]
impl StyksMakeSupplier {
    pub fn init(&mut self) {
        // Grant the admin role to the contract deployer.
        let deployer = self.env().caller();
        let admin_role = SupplierRole::Admin.role_id();
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

    pub fn update_public_key(&mut self, public_key: PublicKey) {
        // Make sure only ConfigManager can update the public key.
        self.assert_config_manager(self.env().caller());

        // Update the public key.
        let mut config = self.get_config();
        config.public_key = public_key;
        self.config.set(config);
    }

    pub fn set_config(&mut self, config: MakeSupplierConfig) {
        // Make sure only ConfigManager can set the config.
        self.assert_config_manager(self.env().caller());

        // Update the config.
        self.config.set(config);
    }

    pub fn get_config(&self) -> MakeSupplierConfig {
        self.config
            .get_or_revert_with(StyksSupplierError::ConfigNotSet)
    }

    pub fn get_config_or_none(&self) -> Option<MakeSupplierConfig> {
        self.config.get()
    }

    /// Verifies the signature against the data.
    pub fn report_signed_prices(&mut self, signature: Bytes, data: Bytes) {
        let config = self.get_config();

        let parsed_data = styks_make_parser::parse_data(&data)
            .map_err(StyksSupplierError::from)
            .unwrap_or_revert(self);

        // Verify the signature.
        self.assert_valid_signature(signature, data);

        let timestamp = parsed_data
            .timestamp()
            .map_err(StyksSupplierError::from)
            .unwrap_or_revert(self);

        // Verify the timestamp.
        self.assert_timestamp_in_range(timestamp, config.timestamp_tolerance);

        // Load the price feed.
        let mut feed = StyksPriceFeedContractRef::new(self.env(), config.price_feed_address);

        // Load the PriceFeedId.
        let price_feed_id = match config.price_feed_id(&parsed_data.currency_id) {
            Some(id) => id.to_owned(),
            None => self.env().revert(StyksSupplierError::PriceFeedIdNotFound),
        };

        // Report the price to the feed.
        feed.add_to_feed(vec![(price_feed_id, parsed_data.price)]);
    }
}

impl StyksMakeSupplier {
    fn assert_role(&self, address: &Address, role: SupplierRole) {
        if !self.has_role(&role.role_id(), address) {
            let error = match role {
                SupplierRole::Admin => StyksSupplierError::NotAdminRole,
                SupplierRole::ConfigManager => StyksSupplierError::NotConfigManagerRole,
            };
            self.env().revert(error);
        }
    }

    fn assert_config_manager(&self, address: Address) {
        self.assert_role(&address, SupplierRole::ConfigManager);
    }

    fn assert_valid_signature(&self, signature: Bytes, data: Bytes) {
        let pk = self.get_config().public_key;
        let hashed_data = Sha256::digest(&data);
        let signature = odra::casper_types::crypto::Signature::from_bytes(&signature.to_vec())
            .unwrap_or_revert_with(self, StyksSupplierError::InvalidSignature)
            .0;
        let result = odra::casper_types::crypto::verify(hashed_data, &signature, &pk).is_ok();

        if !result {
            self.env().revert(StyksSupplierError::BadSignature);
        }
    }

    fn assert_timestamp_in_range(&self, reported: u64, tolerance: u64) {
        let current_time = self.env().get_block_time_secs();
        if reported < current_time.saturating_sub(tolerance) || reported > current_time + tolerance
        {
            self.env().revert(StyksSupplierError::TimestampOutOfRange);
        }
    }
}

#[cfg(test)]
mod tests {
    use odra::host::{Deployer, HostEnv, NoArgs};
    use styks_make_parser::output::SignedPriceData;

    use crate::styks_price_feed::{
        StyksPriceFeed, StyksPriceFeedConfig, StyksPriceFeedHostRef, StyksPriceFeedRole,
    };

    use super::*;

    fn setup() -> (
        HostEnv,
        StyksPriceFeedHostRef,
        StyksMakeSupplierHostRef,
        MakeSupplierConfig,
        SignedPriceData,
    ) {
        let env = odra_test::env();
        let admin = env.get_account(0);

        // Load SignedPriceData from file.
        let signed_data = styks_make_parser::test_utils::test_data();

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
        let mut supplier = StyksMakeSupplier::deploy(&env, NoArgs);
        let supplier_config = MakeSupplierConfig {
            public_key: signed_data.public_key().unwrap(),
            feed_ids: vec![(String::from("1"), String::from("CSPRUSD"))],
            price_feed_address: feed.address(),
            timestamp_tolerance: 1, // 1 sec tolerance
        };
        supplier.grant_role(&SupplierRole::ConfigManager.role_id(), &admin);
        supplier.set_config(supplier_config.clone());

        // Allow StyksMakeSupplier to add prices to StyksPriceFeed.
        let role = StyksPriceFeedRole::PriceSupplier.role_id();
        feed.grant_role(&role, &supplier.address());

        (env, feed, supplier, supplier_config, signed_data)
    }

    #[test]
    fn test_styks_supplier() {
        let (env, feed, mut supplier, supplier_config, signed_data) = setup();
        let id = supplier_config.feed_ids[0].1.clone();

        // Check initial config.
        assert_eq!(supplier.get_config(), supplier_config);

        // Assuming the test starts at block time 1000.
        let timestamp = 1767993960;
        env.advance_block_time(timestamp * 1000);
        assert_eq!(timestamp, env.block_time_secs());

        // Price should be empty initially.
        assert_eq!(feed.get_twap_price(&id), None);
        let d = styks_make_parser::parse_data(&signed_data.payload_bytes()).unwrap();
        println!("{}", d.timestamp().unwrap());

        println!("Reporting signed prices...");
        // Report signed prices.
        supplier.report_signed_prices(
            Bytes::from(signed_data.signature_bytes().unwrap()),
            Bytes::from(signed_data.payload_bytes()),
        );

        // Check the reported price.
        let price = feed.get_twap_price(&id);
        assert_eq!(price, Some(501611));
    }
}
