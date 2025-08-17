use odra::host::HostEnv;
use odra_cli::{
    cspr, scenario::{Args, Error, Scenario, ScenarioMetadata}, CommandArg, ContractProvider, DeployedContractsContainer
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
            heartbeat_interval: 10 * 60,
            heartbeat_tolerance: 30,
            twap_window: 3,
            twap_tolerance: 1,
            price_feed_ids: vec![String::from("CSPRUSD")],
        };

        odra_cli::log("Setting configuration.");
        env.set_gas(cspr!(5));
        contract.set_config(config);
        Ok(())
    }
}
