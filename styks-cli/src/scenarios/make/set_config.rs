use odra::{contract_def::HasIdent, host::HostEnv};
use odra_cli::{
    cspr,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    ContractProvider, DeployedContractsContainer,
};
use styks_contracts::{
    make_supplier::{MakeSupplierConfig, StyksMakeSupplier},
    styks_price_feed::{StyksPriceFeed, StyksPriceFeedConfig},
};
use styks_make_parser::test_utils;

pub struct SetConfig;

impl ScenarioMetadata for SetConfig {
    const NAME: &'static str = "SetConfig";
    const DESCRIPTION: &'static str = "Sets the configuration for the StyksPriceFeed contract.";
}

impl Scenario for SetConfig {
    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> Result<(), Error> {
        self.configure_feed(env, container)?;
        self.configure_supplier(env, container)?;
        Ok(())
    }
}

impl SetConfig {
    fn configure_feed(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
    ) -> Result<(), Error> {
        // Configuring the StyksPriceFeed contract.
        odra_cli::log("Setting configuration for StyksPriceFeed contract.");
        let mut feed = container.contract_ref::<StyksPriceFeed>(&env)?;
        let config = StyksPriceFeedConfig {
            heartbeat_interval: 30 * 60,
            heartbeat_tolerance: 60,
            // heartbeat_interval: 60,
            // heartbeat_tolerance: 20,
            twap_window: 3,
            twap_tolerance: 1,
            price_feed_ids: vec![String::from("CSPRUSD")],
        };

        if let Some(current_config) = feed.get_config_or_none() {
            if current_config == config {
                odra_cli::log("Configuration is already set to the desired values.");
                return Ok(());
            }
        }
        odra_cli::log("Current configuration does not match the desired values.");
        env.set_gas(cspr!(4));
        feed.set_config(config);
        odra_cli::log("Configuration set successfully for StyksPriceFeed contract.");
        Ok(())
    }

    fn configure_supplier(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
    ) -> Result<(), Error> {
        // Configuring the MakeSupplier contract.
        odra_cli::log("Setting configuration for MakeSupplier contract.");
        let mut supplier = container.contract_ref::<StyksMakeSupplier>(&env)?;
        let feed_addr = container.address_by_name(&StyksPriceFeed::ident()).unwrap();

        // Load blocky configuration.
        let signed_data = test_utils::test_data();

        let supplier_config = MakeSupplierConfig {
            public_key: signed_data.public_key().unwrap(),
            feed_ids: vec![(String::from("1"), String::from("CSPRUSD"))],
            price_feed_address: feed_addr,
            timestamp_tolerance: 20 * 60, // 20 minutes tolerance
        };

        if let Some(current_config) = supplier.get_config_or_none() {
            if current_config == supplier_config {
                odra_cli::log("The supplier configuration is already set to the desired values.");
                return Ok(());
            } else {
                odra_cli::log("Current configuration does not match the desired values.");
            }
        } else {
            odra_cli::log("The supplier configuration is not set, setting it now.");
        }

        env.set_gas(cspr!(3.5));
        supplier.set_config(supplier_config);
        odra_cli::log("Configuration set successfully for the supplier contract.");

        Ok(())
    }
}
