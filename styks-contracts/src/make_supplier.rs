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
    staking::StakingRewardsContractRef, styks_price_feed::StyksPriceFeedContractRef,
    supplier_error::StyksSupplierError, supplier_role::SupplierRole,
};

mod v2;

/// Minimum seconds between two accepted reports from the same provider (5 min).
/// Hardcoded — see `Stysk2.0.md` defaults reference.
const PER_PROVIDER_RATE_LIMIT: u64 = 300;

// --- Configuration ---
#[odra::odra_type]
pub struct MakeSupplierConfig {
    pub public_key: PublicKey,
    pub feed_ids: Vec<(String, PriceFeedId)>, // (make_id, price_feed_id)
    pub price_feed_address: Address,
    /// `StakingRewards` contract reporters are credited against. The supplier
    /// must hold the `Reporter` role there.
    pub staking_rewards_address: Address,
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
    /// Block time (secs) of each provider's last accepted report.
    last_report: Mapping<Address, u64>,
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

    /// Block time (secs) of `provider`'s last accepted report, or `None`.
    pub fn get_last_report(&self, provider: &Address) -> Option<u64> {
        self.last_report.get(provider)
    }

    /// Verify a Make-signed price payload, forward it to the feed and credit the
    /// caller in `StakingRewards`.
    ///
    /// Flow (see `Stysk2.0.md` reporting flow):
    /// 1. Verify the Make signature over `data`.
    /// 2. Parse the payload → (currency_id, price, attest timestamp).
    /// 3. Per-provider rate limit: reject if the caller reported < 300s ago.
    /// 4. Push the price to the feed.
    /// 5. Credit the caller in `StakingRewards`.
    ///
    /// The reporter is `env().caller()` — the credited provider. There is no
    /// registration check here: unregistered relayers can keep the price fresh,
    /// they just can't claim. The supplier's own state write (the rate-limit
    /// stamp) happens before the two outbound calls (checks-effects-interactions).
    pub fn report_signed_prices(&mut self, signature: Bytes, data: Bytes) {
        let config = self.get_config();
        let caller = self.env().caller();

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

        // Per-provider rate limit (uses the on-chain clock, not the attest time).
        let now = self.env().get_block_time_secs();
        let last = self.last_report.get(&caller);
        if let Some(last) = last {
            if now < last + PER_PROVIDER_RATE_LIMIT {
                self.env().revert(StyksSupplierError::RateLimited);
            }
        }
        self.last_report.set(&caller, now);

        // Load the PriceFeedId.
        let price_feed_id = match config.price_feed_id(&parsed_data.currency_id) {
            Some(id) => id.to_owned(),
            None => self.env().revert(StyksSupplierError::PriceFeedIdNotFound),
        };

        // Report the price to the feed.
        let mut feed = StyksPriceFeedContractRef::new(self.env(), config.price_feed_address);
        feed.add_to_feed(vec![(price_feed_id, parsed_data.price)]);

        // Credit the caller in StakingRewards for epoch reward accounting.
        let mut staking =
            StakingRewardsContractRef::new(self.env(), config.staking_rewards_address);
        staking.record_report(caller);
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

    use crate::staking::{
        StakingRewards, StakingRewardsHostRef, StakingRewardsInitArgs, StakingRole,
    };
    use crate::styks_price_feed::{
        StyksPriceFeed, StyksPriceFeedConfig, StyksPriceFeedHostRef, StyksPriceFeedRole,
    };
    use crate::token::StyksToken;

    use super::*;

    fn setup() -> (
        HostEnv,
        StyksPriceFeedHostRef,
        StyksMakeSupplierHostRef,
        StakingRewardsHostRef,
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

        // Deploy StyksToken + StakingRewards.
        let token = StyksToken::deploy(&env, NoArgs);
        let staking = StakingRewards::deploy(
            &env,
            StakingRewardsInitArgs {
                token: token.address(),
            },
        );

        // Deploy StyksMakeSupplier contract.
        let mut supplier = StyksMakeSupplier::deploy(&env, NoArgs);
        let supplier_config = MakeSupplierConfig {
            public_key: signed_data.public_key().unwrap(),
            feed_ids: vec![(String::from("1"), String::from("CSPRUSD"))],
            price_feed_address: feed.address(),
            staking_rewards_address: staking.address(),
            timestamp_tolerance: 1, // 1 sec tolerance
        };
        supplier.grant_role(&SupplierRole::ConfigManager.role_id(), &admin);
        supplier.set_config(supplier_config.clone());

        // Allow StyksMakeSupplier to add prices to StyksPriceFeed.
        feed.grant_role(
            &StyksPriceFeedRole::PriceSupplier.role_id(),
            &supplier.address(),
        );

        // Allow StyksMakeSupplier to credit reports in StakingRewards.
        let mut staking = staking;
        staking.grant_role(&StakingRole::Reporter.role_id(), &supplier.address());

        (env, feed, supplier, staking, supplier_config, signed_data)
    }

    #[test]
    fn test_styks_supplier() {
        let (env, feed, mut supplier, staking, supplier_config, signed_data) = setup();
        let id = supplier_config.feed_ids[0].1.clone();
        let reporter = env.get_account(0);

        // Check initial config.
        assert_eq!(supplier.get_config(), supplier_config);

        // Assuming the test starts at block time 1000.
        let timestamp = 1767993960;
        env.advance_block_time(timestamp * 1000);
        assert_eq!(timestamp, env.block_time_secs());

        // Price should be empty initially.
        assert_eq!(feed.get_twap_price(&id), None);

        // Report signed prices.
        supplier.report_signed_prices(
            Bytes::from(signed_data.signature_bytes().unwrap()),
            Bytes::from(signed_data.payload_bytes()),
        );

        // Check the reported price.
        assert_eq!(feed.get_twap_price(&id), Some(501));

        // The caller was credited a report in the current staking epoch.
        let epoch = staking.current_epoch();
        assert_eq!(staking.get_epoch_reports(epoch, &reporter), 1);
        assert_eq!(supplier.get_last_report(&reporter), Some(timestamp));
    }

    #[test]
    fn test_per_provider_rate_limit() {
        let (env, _feed, mut supplier, _staking, _config, signed_data) = setup();

        // Widen the timestamp tolerance so the (fixed) attest time keeps
        // validating as block time advances — we want to isolate the rate limit.
        let mut cfg = supplier.get_config();
        cfg.timestamp_tolerance = 10_000;
        supplier.set_config(cfg);

        let timestamp = 1767993960;
        env.advance_block_time(timestamp * 1000);

        let signature = Bytes::from(signed_data.signature_bytes().unwrap());
        let payload = Bytes::from(signed_data.payload_bytes());

        // First report is accepted.
        supplier.report_signed_prices(signature.clone(), payload.clone());

        // A second report within 300s from the same caller is rejected.
        env.advance_block_time(299 * 1000);
        assert_eq!(
            supplier.try_report_signed_prices(signature.clone(), payload.clone()),
            Err(StyksSupplierError::RateLimited.into())
        );

        // Once the 300s window elapses it is accepted again.
        env.advance_block_time(1 * 1000);
        supplier.report_signed_prices(signature, payload);
    }
}
