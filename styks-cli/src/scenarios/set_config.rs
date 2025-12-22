use odra::{casper_types::bytesrepr::Bytes, contract_def::HasIdent, host::HostEnv};
use odra_cli::{
    cspr, scenario::{Args, Error, Scenario, ScenarioMetadata}, CommandArg, ContractProvider, DeployedContractsContainer
};
use styks_blocky_parser::{block_output_for_tests, wasm_hash_for_tests};
use styks_contracts::{styks_blocky_supplier::{StyksBlockySupplerConfig, StyksBlockySupplier}, styks_price_feed::{StyksPriceFeed, StyksPriceFeedConfig}};

pub struct SetConfig;

impl ScenarioMetadata for SetConfig {
    const NAME: &'static str = "SetConfig";
    const DESCRIPTION: &'static str = "Sets the configuration for the StyksPriceFeed contract.";
}

impl Scenario for SetConfig {
    fn args(&self) -> Vec<CommandArg> { vec![] }

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
    fn configure_feed(&self, env: &HostEnv, container: &DeployedContractsContainer) -> Result<(), Error> {
        // Configuring the StyksPriceFeed contract.
        odra_cli::log("Setting configuration for StyksPriceFeed contract.");
        let mut feed = container.contract_ref::<StyksPriceFeed>(&env)?;
        let config = StyksPriceFeedConfig {
            heartbeat_interval: 30 * 60,  // 30 minutes
            heartbeat_tolerance: 60,
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
        // Configuring the StyksBlockySupplier contract.
        odra_cli::log("Setting configuration for StyksBlockySupplier contract.");
        let mut supplier = container.contract_ref::<StyksBlockySupplier>(&env)?;
        let feed_addr = container.address_by_name(&StyksPriceFeed::ident()).unwrap();

        // Load blocky configuration.
        let wasm_hash = wasm_hash_for_tests();
        let blocky_output = block_output_for_tests();
        let public_key = blocky_output.public_key_bytes();

        let supplier_config = StyksBlockySupplerConfig {
            wasm_hash,
            coingecko_feed_ids: vec![
                (String::from("Gate_CSPR_USD"), String::from("CSPRUSD"))
            ],
            price_feed_address: feed_addr,
            timestamp_tolerance: 20 * 60 // 20 minutes tolerance
        };
        
        let config_needs_update = if let Some(current_config) = supplier.get_config_or_none() {
            if current_config == supplier_config {
                odra_cli::log("StyksBlockySupplier configuration is already set to the desired values.");
                false
            } else {
                odra_cli::log("Current configuration does not match the desired values.");
                true
            }
        } else {
            odra_cli::log("StyksBlockySupplier configuration is not set, setting it now.");
            true
        };

        if config_needs_update {
            env.set_gas(cspr!(3.5));
            supplier.set_config(supplier_config);
            odra_cli::log("Configuration set successfully for StyksBlockySupplier contract.");
        }

        // Auto-bootstrap key ring with current public key (enables rotation immediately)
        let keys = supplier.get_signer_keys();
        if keys.is_empty() {
            odra_cli::log("Auto-bootstrapping key ring with current public key.");
            env.set_gas(cspr!(3));
            supplier.add_signer_key(Bytes::from(public_key), 0, 0);
            odra_cli::log("Key ring bootstrapped successfully.");
        } else {
            odra_cli::log("Key ring already has keys, skipping bootstrap.");
        }

        // Default expected function name to 'priceFunc' for new deployments
        let expected_fn = supplier.get_expected_function();
        if expected_fn.is_empty() {
            odra_cli::log("Setting expected function name to 'priceFunc' (default).");
            env.set_gas(cspr!(2.5));
            supplier.set_expected_function("priceFunc".to_string());
            odra_cli::log("Expected function name set successfully.");
        } else {
            odra_cli::log(format!("Expected function already set to: {}", expected_fn));
        }

        Ok(())
    }
}