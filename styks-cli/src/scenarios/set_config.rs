use odra::host::HostEnv;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};
use styks_contracts::styks_price_feed::{StyksPriceFeed, StyksPriceFeedConfig};

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
    ) -> core::result::Result<(), Error> {
        let mut contract = container.contract_ref::<StyksPriceFeed>(&env)?;
        let config = StyksPriceFeedConfig {
            heartbeat_interval: 60,
            heartbeat_tolerance: 20,
            twap_window: 5,
            twap_tolerance: 3,
            price_feed_ids: vec![String::from("CSPRUSD")],
        };

        odra_cli::log(format!("Setting configuration."));
        env.set_gas(10_000_000_000);
        contract.set_config(config);
        Ok(())
    }
}
