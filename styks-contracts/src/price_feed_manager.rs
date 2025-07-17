use odra::prelude::*;
use odra_modules::access::{AccessControl, Role, DEFAULT_ADMIN_ROLE};

use crate::price_feed::{PriceFeed, PriceFeedConfig, PriceFeedId, PriceRecord};
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
    price_feeds: Mapping<PriceFeedId, PriceFeed>
}

#[odra::module]
impl PriceFeedManager {
    pub fn init(&mut self) {}

    delegate! {
        to self.access_control {
            fn has_role(&self, role: &Role, address: &Address) -> bool;
            fn grant_role(&mut self, role: &Role, address: &Address);
            fn revoke_role(&mut self, role: &Role, address: &Address);
            fn get_role_admin(&self, role: &Role) -> Role;
            fn renounce_role(&mut self, role: &Role, address: &Address);
        }
    }

    pub fn add_price(&mut self, config: PriceFeedConfig) {
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
}

impl PriceFeedManager {
    fn assert_role(&self, address: &Address, role: ContractRole, error: PriceFeedError) {
        if !self.has_role(&role.role_id(), address) {
            self.env().revert(error);
        }
    }

    fn assert_admin_role(&self, address: &Address) {
        self.assert_role(address, ContractRole::Admin, PriceFeedError::NotAdminRole);
    }

    fn assert_price_feed_manager_role(&self, address: &Address) {
        self.assert_role(address, ContractRole::PriceFeedManager, PriceFeedError::NotPriceFeedManagerRole);
    }

    fn assert_price_supplier_role(&self, address: &Address) {
        self.assert_role(address, ContractRole::PriceSuppliers, PriceFeedError::NotPriceSupplierRole);
    }

    pub fn is_price_feed_initialized(&self, price_feed_id: &PriceFeedId) -> bool {
        self.price_feed_initialized.get(price_feed_id).unwrap_or_default()
    }
}