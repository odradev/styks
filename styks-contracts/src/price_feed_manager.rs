use odra::prelude::*;
use odra_modules::access::{AccessControl, Role, DEFAULT_ADMIN_ROLE};

use crate::price_feed::{Price, PriceFeed, PriceFeedConfig, PriceFeedId, PriceRecord, TimestampSec};
use crate::error::PriceFeedError;

pub enum ContractRole {
    Admin,
    PriceFeedManager,
    PriceSuppliers
}

impl ContractRole {
    pub fn role_id(&self) -> Role {
        match self {
            ContractRole::Admin => DEFAULT_ADMIN_ROLE,
            ContractRole::PriceFeedManager => [1u8; 32],
            ContractRole::PriceSuppliers => [2u8; 32],
        }
    }
}

#[odra::module]
pub struct PriceFeedManager {
    access_control: SubModule<AccessControl>,
    price_feed_initialized: Mapping<PriceFeedId, bool>,
    price_feeds: Mapping<PriceFeedId, PriceFeed>,
}

#[odra::module]
impl PriceFeedManager {
    pub fn init(&mut self) {
        // Grant the admin role to the contract deployer.
        let deployer = self.env().caller();
        self.access_control.unchecked_grant_role(&ContractRole::Admin.role_id(), &deployer);
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

    pub fn add_price_feed(&mut self, config: PriceFeedConfig) {
        // Ensure the caller has the PriceFeedManager role
        self.assert_price_feed_manager_role(&self.env().caller());

        let price_feed_id = config.price_feed_id.clone();
        // Ensure the price feed does not already exist
        if self.is_price_feed_initialized(&price_feed_id) {
            self.env().revert(PriceFeedError::PriceFeedAlreadyExists);
        }

        // Initialize the price feed with the provided configuration.
        self.price_feeds.module(&price_feed_id).set_config(config);

        // Mark the price feed as initialized.
        self.price_feed_initialized.set(&price_feed_id, true);
    }

    pub fn publish_price(
        &mut self,
        price_feed_id: &PriceFeedId,
        price: Price,
        timestamp: TimestampSec,
    ) {
        // Ensure the caller has the PriceSupplier role
        self.assert_price_supplier_role(&self.env().caller());

        // Post the data to the price feed.
        self.price_feed(price_feed_id).publish_price(PriceRecord {
            timestamp,
            price,
        });
    }

    // --- Getters ---

    pub fn get_price(&self, price_feed_id: &PriceFeedId) -> Option<Price> {
        self.price_feed(price_feed_id).get_twap_price()
    }

    pub fn get_price_feed_history_counter(&self, price_feed_id: &PriceFeedId) -> u64 {
        self.price_feed(price_feed_id).get_price_history_counter()
    }

    pub fn get_price_history(
        &self,
        price_feed_id: &PriceFeedId,
        id: u64
    ) -> Option<PriceRecord> {
        self.price_feed(price_feed_id).get_price_history(id)
    }

    pub fn is_price_feed_initialized(&self, price_feed_id: &PriceFeedId) -> bool {
        self.price_feed_initialized.get(price_feed_id).unwrap_or_default()
    }
}

impl PriceFeedManager {
    fn assert_role(&self, address: &Address, role: ContractRole, error: PriceFeedError) {
        if !self.has_role(&role.role_id(), address) {
            self.env().revert(error);
        }
    }

    // fn assert_admin_role(&self, address: &Address) {
    //     self.assert_role(address, ContractRole::Admin, PriceFeedError::NotAdminRole);
    // }

    fn assert_price_feed_manager_role(&self, address: &Address) {
        self.assert_role(address, ContractRole::PriceFeedManager, PriceFeedError::NotPriceFeedManagerRole);
    }

    fn assert_price_supplier_role(&self, address: &Address) {
        self.assert_role(address, ContractRole::PriceSuppliers, PriceFeedError::NotPriceSupplierRole);
    }

    pub fn assert_feed_exists(&self, price_feed_id: &PriceFeedId) {
        if !self.is_price_feed_initialized(price_feed_id) {
            self.env().revert(PriceFeedError::PriceFeedNotFound);
        }
    }

    fn price_feed(&self, price_feed_id: &PriceFeedId) -> SubModule<PriceFeed> {
        if !self.is_price_feed_initialized(price_feed_id) {
            self.env().revert(PriceFeedError::PriceFeedNotFound);
        }
        self.price_feeds.module(price_feed_id)
    }
}

#[cfg(test)]
mod tests {
    use odra::host::{Deployer, NoArgs};

    use crate::minutes;

    use super::*;

    #[test]
    fn test_price_feed_manager() {
        // Given new contract.
        let env = odra_test::env();
        env.advance_block_time(86400_000);

        let mut price_feed_manager = PriceFeedManager::deploy(&env, NoArgs);
        // let deployer = env.get_account(0);

        // Alice as price feed manager.
        let alice = env.get_account(1);
        price_feed_manager.grant_role(&ContractRole::PriceFeedManager.role_id(), &alice);

        // Bob as price supplier.
        let bob = env.get_account(2);
        price_feed_manager.grant_role(&ContractRole::PriceSuppliers.role_id(), &bob);

        // And a price feed configuration.
        let price_feed_id = PriceFeedId::from("BTCUSD");
        let heartbeat = minutes(10);
        let twap_window = minutes(30);
        let new_data_timeout = minutes(2);
        let config = PriceFeedConfig {
            price_feed_id,
            heartbeat,
            twap_window,
            new_data_timeout,
        };

        // When adding a new price feed.
        env.set_caller(alice);
        price_feed_manager.add_price_feed(config.clone());

        // Then it's possible to record a price.
        env.set_caller(bob);
        let price1 = 50000;
        let timestamp1 = env.block_time_secs() - 1;
        price_feed_manager.publish_price(&config.price_feed_id, price1, timestamp1);

        // And the price is retrievable.
        let price = price_feed_manager.get_price(&config.price_feed_id);
        assert_eq!(price, Some(price1));

        // When price is recorded again.
        env.advance_block_time(heartbeat * 1000);
        let price2 = 51000;
        let timestamp2 = env.block_time_secs() - 1 ;
        price_feed_manager.publish_price(&config.price_feed_id, price2, timestamp2);

        // Then the new price is retrievable.
        let _price = price_feed_manager.get_price(&config.price_feed_id);
        // assert_eq!(price, Some(price2)); // TODO: Correct it.

        // When price is recorded again.
        env.advance_block_time(heartbeat * 1000);
        let price3 = 60000;
        let timestamp3 = env.block_time_secs() - 1;
        price_feed_manager.publish_price(&config.price_feed_id, price3, timestamp3);
        
        // Then the new price is retrievable.
        let _price = price_feed_manager.get_price(&config.price_feed_id);
        // assert_eq!(price, Some(price3)); // TODO: Correct it.

        // All previous prices are still retrievable.
        let history_counter = price_feed_manager.get_price_feed_history_counter(&config.price_feed_id);
        assert_eq!(history_counter, 3);
        
        let record1 = price_feed_manager.get_price_history(&config.price_feed_id, 0).unwrap();
        assert_eq!(record1.price, price1);
        assert_eq!(record1.timestamp, timestamp1);
        let record2 = price_feed_manager.get_price_history(&config.price_feed_id, 1).unwrap();
        assert_eq!(record2.price, price2);
        assert_eq!(record2.timestamp, timestamp2);
        let record3 = price_feed_manager.get_price_history(&config.price_feed_id, 2).unwrap();
        assert_eq!(record3.price, price3);
        assert_eq!(record3.timestamp, timestamp3);
    }
}