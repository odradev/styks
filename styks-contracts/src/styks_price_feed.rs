use odra::prelude::*;
use odra_modules::access::{AccessControl, Role, DEFAULT_ADMIN_ROLE};

use styks_core::{
    heartbeat::{Heartbeat, HeartbeatError},
    twap::{TWAPError, TWAP},
    Price, PriceFeedId,
};

// --- Errors ---

#[odra::odra_error]
pub enum StyksPriceFeedError {
    // Config errors.
    ConfigNotSet = 45000,
    HeartbeatIntervalShouldBeGreaterThanZero = 45001,
    HeartbeatTolaranceShouldBeLessThanHalfOfInterval = 45002,
    TWAPWindowCannotBeZero = 45003,
    TWAPToleranceMustBeLessThanWindow = 45004,
    TWAPTooManyValues = 45005,
    PriceFeedIdIsEmptyString = 45006,
    PriceFeedIdNotUnique = 45007,

    // Role errors.
    NotAdminRole = 45010,
    NotConfigManagerRole = 45011,
    NotPriceSupplierRole = 45012,

    // Add to feed errors.
    NotInHeartbeatWindow = 45020,
    FeedAlreadyUpdatedInCurrentHeartbeatWindow = 45021,
    PriceFeedIdsMissmatch = 45022,
}

impl From<HeartbeatError> for StyksPriceFeedError {
    fn from(error: HeartbeatError) -> Self {
        use HeartbeatError::*;
        use StyksPriceFeedError::*;
        match error {
            TolaranceShouldBeLessThanHalfOfInterval => {
                HeartbeatTolaranceShouldBeLessThanHalfOfInterval
            }
            IntervalShouldBeGreaterThanZero => HeartbeatIntervalShouldBeGreaterThanZero,
        }
    }
}

impl From<TWAPError> for StyksPriceFeedError {
    fn from(error: TWAPError) -> Self {
        use StyksPriceFeedError::*;
        use TWAPError::*;
        match error {
            WindowCannotBeZero => TWAPWindowCannotBeZero,
            ToleranceMustBeLessThanWindow => TWAPToleranceMustBeLessThanWindow,
            TooManyValues => TWAPTooManyValues,
        }
    }
}

// --- Access Control Roles ---

pub enum StyksPriceFeedRole {
    Admin,
    ConfigManager,
    PriceSupplier,
}

impl StyksPriceFeedRole {
    pub fn role_id(&self) -> Role {
        match self {
            StyksPriceFeedRole::Admin => DEFAULT_ADMIN_ROLE,
            StyksPriceFeedRole::ConfigManager => [1u8; 32],
            StyksPriceFeedRole::PriceSupplier => [2u8; 32],
        }
    }
}

// --- Configuration ---

#[odra::odra_type]
pub struct StyksPriceFeedConfig {
    pub heartbeat_interval: u64,
    pub heartbeat_tolerance: u64,
    pub twap_window: u32,
    pub twap_tolerance: u32,
    pub price_feed_ids: Vec<PriceFeedId>,
}

impl StyksPriceFeedConfig {
    pub fn validate(&self) -> Result<(), StyksPriceFeedError> {
        // Create Heartbeat to validate heartbeat parameters.
        let heartbeat = Heartbeat::new(
            0, // Current time is not relevant for validation.
            self.heartbeat_interval,
            self.heartbeat_tolerance,
        );
        if let Err(error) = heartbeat {
            return Err(StyksPriceFeedError::from(error));
        };

        // Create TWAP to validate TWAP parameters.
        let twap = TWAP::new(self.twap_window, self.twap_tolerance, Vec::new());
        if let Err(error) = twap {
            return Err(StyksPriceFeedError::from(error));
        };

        // Validate PriceFeedIds. Make sure all IDs are unique and not empty.
        let mut seen_ids = BTreeMap::new();
        for id in &self.price_feed_ids {
            if id.is_empty() {
                return Err(StyksPriceFeedError::PriceFeedIdIsEmptyString);
            }
            if seen_ids.insert(id.clone(), ()).is_some() {
                return Err(StyksPriceFeedError::PriceFeedIdNotUnique);
            }
        }

        Ok(())
    }

    pub fn sorted_price_feed_ids(&self) -> Vec<PriceFeedId> {
        let mut ids = self.price_feed_ids.clone();
        ids.sort();
        ids
    }
}

// --- Styks Price Feed Smart Contract ---

#[odra::module]
pub struct StyksPriceFeed {
    access_control: SubModule<AccessControl>,
    config: Var<StyksPriceFeedConfig>,
    last_heartbeat: Var<Option<u64>>,
    twap_store: Mapping<PriceFeedId, Vec<Option<Price>>>,
}

#[odra::module]
impl StyksPriceFeed {
    pub fn init(&mut self) {
        // Grant the admin role to the contract deployer.
        let deployer = self.env().caller();
        let admin_role = StyksPriceFeedRole::Admin.role_id();
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

    pub fn set_config(&mut self, config: StyksPriceFeedConfig) {
        // Make sure only ConfigManager can set the config.
        self.assert_config_manager(&self.env().caller());

        // Validate the config.
        config.validate().unwrap_or_revert(&self.env());

        // Update the config.
        self.config.set(config);
    }

    pub fn get_config(&self) -> StyksPriceFeedConfig {
        self.config
            .get()
            .unwrap_or_revert_with(&self.env(), StyksPriceFeedError::ConfigNotSet)
    }

    pub fn get_current_twap_store(&self, id: &PriceFeedId) -> Vec<Option<Price>> {
        self.twap_store.get(id).unwrap_or_default()
    }

    pub fn get_last_heartbeat(&self) -> Option<u64> {
        self.last_heartbeat.get_or_default()
    }

    pub fn add_to_feed(&mut self, input: Vec<(PriceFeedId, Price)>) {
        // Make sure only PriceSupplier can add prices.
        self.assert_price_supplier(&self.env().caller());

        // Load configuration.
        let config = self.get_config();

        // Create Heartbeat object.
        let heartbeat = Heartbeat::new(
            self.env().get_block_time_secs(),
            config.heartbeat_interval,
            config.heartbeat_tolerance,
        )
        .map_err(StyksPriceFeedError::from)
        .unwrap_or_revert(&self.env());

        // Load the current heartbeat state.
        let heartbeat_state = heartbeat.current_state();
        
        // Extract the current heartbeat time or revert if not in a heartbeat window.
        let current_heartbeat_time = if let Some(current_window) = heartbeat_state.current {
            current_window.middle
        } else {
            self.env().revert(StyksPriceFeedError::NotInHeartbeatWindow);
        };

        // Load the last recorded heartbeat time.
        let last_heartbeat = self.last_heartbeat.get_or_default();

        // Extract the number of missed heartbeats since the last recorded heartbeat.
        let missed_heartbeats = if let Some(time) = last_heartbeat {
            // Revert if the feed was already updated in the current heartbeat window.
            if time == current_heartbeat_time {
                self.env()
                    .revert(StyksPriceFeedError::FeedAlreadyUpdatedInCurrentHeartbeatWindow);
            }
            heartbeat.count_missed_heartbeats_since(time)
        } else {
            // If no last heartbeat, assume no missed heartbeats.
            0
        };

        // Check if all PriceFeedIds are present in input.
        let expected_ids: Vec<String> = config.sorted_price_feed_ids();
        let input_ids: Vec<String> = input.iter().map(|(id, _)| id.clone()).collect();
        if input_ids != expected_ids {
            self.env().revert(StyksPriceFeedError::PriceFeedIdsMissmatch);
        }

        // Update the TWAP store with the new prices.
        for (id, price) in input {
            let twap_prices = self.twap_store.get(&id).unwrap_or_default();
            let mut twap = TWAP::new(
                config.twap_window,
                config.twap_tolerance,
                twap_prices,
            )
            .map_err(StyksPriceFeedError::from)
            .unwrap_or_revert(&self.env());

            // Add missed heartbeats to the TWAP.
            for _ in 0..missed_heartbeats {
                twap.add_missed_value(); // Add None for missed heartbeats.
            }

            // Add the new price to the TWAP.
            twap.add_value(price);

            // Store the updated TWAP prices.
            self.twap_store.set(&id, twap.values());
        }

        // Update the last heartbeat time to the current heartbeat time.
        self.last_heartbeat.set(Some(current_heartbeat_time));
    }

    pub fn get_twap_price(&self, id: &PriceFeedId) -> Option<Price> {
        // Load configuration.
        let config = self.get_config();

        // Create Heartbeat object.
        let heartbeat = Heartbeat::new(
            self.env().get_block_time_secs(),
            config.heartbeat_interval,
            config.heartbeat_tolerance,
        )
        .map_err(StyksPriceFeedError::from)
        .unwrap_or_revert(&self.env());

        // Load the last recorded heartbeat time.
        let last_heartbeat = self.last_heartbeat.get_or_default().unwrap_or_default();

        // Check how many heartbeats were missed since the last recorded heartbeat.
        let missed_heartbeats = heartbeat.count_missed_heartbeats_since(last_heartbeat);

        let twap_prices = self.twap_store.get(&id).unwrap_or_default();
        let mut twap = TWAP::new(
            config.twap_window,
            config.twap_tolerance,
            twap_prices,
        )
        .map_err(StyksPriceFeedError::from)
        .unwrap_or_revert(&self.env());

        // Add missed heartbeats to the TWAP.
        for _ in 0..missed_heartbeats {
            twap.add_missed_value(); // Add None for missed heartbeats.
        };

        twap.calculate()
    }       
}

impl StyksPriceFeed {
    fn assert_role(&self, address: &Address, role: StyksPriceFeedRole) {
        if !self.has_role(&role.role_id(), address) {
            use StyksPriceFeedError::*;
            use StyksPriceFeedRole::*;
            let error = match role {
                Admin => NotAdminRole,
                ConfigManager => NotConfigManagerRole,
                PriceSupplier => NotPriceSupplierRole,
            };
            self.env().revert(error);
        }
    }

    fn assert_config_manager(&self, address: &Address) {
        self.assert_role(address, StyksPriceFeedRole::ConfigManager);
    }

    fn assert_price_supplier(&self, address: &Address) {
        self.assert_role(address, StyksPriceFeedRole::PriceSupplier);
    }
}

#[cfg(test)]
mod tests {
    use odra::host::{Deployer, HostEnv, NoArgs};

    use super::*;

    fn setup() -> (HostEnv, StyksPriceFeedHostRef, StyksPriceFeedConfig) {
        let env = odra_test::env();
        let admin = env.get_account(0);
        let mut contract = StyksPriceFeed::deploy(&env, NoArgs);
        let config = StyksPriceFeedConfig {
            heartbeat_interval: 100,
            heartbeat_tolerance: 10,
            twap_window: 3,
            twap_tolerance: 1,
            price_feed_ids: vec![String::from("CSPRUSD")],
        };
        contract.grant_role(&StyksPriceFeedRole::ConfigManager.role_id(), &admin);
        contract.grant_role(&StyksPriceFeedRole::PriceSupplier.role_id(), &admin);
        contract.set_config(config.clone());
        (env, contract, config)
    }

    #[test]
    fn test_styks_price_feed() {
        let (env, mut contract, config) = setup();
        let id = config.price_feed_ids[0].clone();

        // Check initial state of the contract.
        assert_eq!(contract.get_config(), config);
        assert!(contract.get_last_heartbeat().is_none());
        assert!(contract.get_current_twap_store(&id).is_empty());
        assert!(contract.get_twap_price(&id).is_none());

        // Assuming the test starts at block time 1000.
        env.advance_block_time(100 * 1000);
        assert_eq!(100, env.block_time_secs()); 

        // --- Heartbeat #1 ---

        // Add a price to the feed.
        contract.add_to_feed(vec![(id.clone(), 1000)]);

        // Check the price.
        assert_eq!(contract.get_twap_price(&id), None);

        // Check the TWAP store.
        let twap_store = contract.get_current_twap_store(&id);
        assert_eq!(twap_store.len(), 1);
        assert_eq!(twap_store[0], Some(1000));

        // Check the last heartbeat.
        assert_eq!(contract.get_last_heartbeat(), Some(100));

        // Move to the middle of the heartbeat window.
        env.advance_block_time(50 * 1000);
        assert_eq!(150, env.block_time_secs());

        // Should not be possible to add price in the middle of the heartbeat window.
        let result = contract.try_add_to_feed(vec![(id.clone(), 1100)]);
        assert_eq!(
            result,
            Err(StyksPriceFeedError::NotInHeartbeatWindow.into())
        );

        assert_eq!(contract.get_twap_price(&id), None);

        // --- Heartbeat #2 ---
        
        // Move to the next heartbeat window.
        env.advance_block_time(40 * 1000);
        assert_eq!(190, env.block_time_secs());

        // Price is still not available.
        assert_eq!(contract.get_twap_price(&id), None);

        // Add a new price to the feed.
        contract.add_to_feed(vec![(id.clone(), 1200)]);

        // Check the price.
        assert_eq!(contract.get_twap_price(&id), Some(1100));

        // Check the TWAP store.
        let twap_store = contract.get_current_twap_store(&id);
        assert_eq!(twap_store.len(), 2);
        assert_eq!(twap_store[0], Some(1000));
        assert_eq!(twap_store[1], Some(1200));

        // Check the last heartbeat.
        assert_eq!(contract.get_last_heartbeat(), Some(200));

        // --- Heartbeat #3 (missed) ---
        // Move to the next heartbeat.
        env.advance_block_time(110 * 1000);
        assert_eq!(300, env.block_time_secs());

        // Check the price.
        assert_eq!(contract.get_twap_price(&id), Some(1100));

        // --- Heartbeat #4 ---
        // Move to the next heartbeat window.
        env.advance_block_time(100 * 1000);

        // Add a new price to the feed.
        contract.add_to_feed(vec![(id.clone(), 1300)]);

        // Check the price.
        assert_eq!(contract.get_twap_price(&id), Some(1250));

        // Check the TWAP store.
        let twap_store = contract.get_current_twap_store(&id);
        assert_eq!(twap_store.len(), 3);
        assert_eq!(twap_store[0], Some(1200));
        assert_eq!(twap_store[1], None);
        assert_eq!(twap_store[2], Some(1300));

        // Check the last heartbeat.
        assert_eq!(contract.get_last_heartbeat(), Some(400));
    }
}
