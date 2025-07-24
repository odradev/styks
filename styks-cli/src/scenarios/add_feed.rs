use odra::{host::HostEnv, schema::casper_contract_schema::NamedCLType};
use odra_cli::{scenario::{Args, Error, Scenario, ScenarioMetadata}, CommandArg, ContractProvider, DeployedContractsContainer};
use styks_contracts::{minutes, price_feed::{PriceFeedConfig}, price_feed_manager::PriceFeedManager};

pub struct AddFeed;

impl ScenarioMetadata for AddFeed {
    const NAME: &'static str = "AddFeed";
    const DESCRIPTION: &'static str = "Adds a CSPR feed to the PriceFeedManager contract.";
}

impl Scenario for AddFeed {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("price_feed_id", "The ID of the price feed to add.", NamedCLType::String)
            .required()
        ]
    }
    
    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args
    ) -> core::result::Result<(), Error> {
        let price_feed_id: String = args.get_single("price_feed_id")?;
        let mut contract = container.contract_ref::<PriceFeedManager>(&env)?;
        
        if contract.is_price_feed_initialized(&price_feed_id) {
            odra_cli::log(format!("{} feed is already initialized.", price_feed_id));
        } else {
            odra_cli::log(format!("Adding {} feed.", price_feed_id));
            env.set_gas(10_000_000_000);
            contract.add_price_feed(PriceFeedConfig {
                price_feed_id,
                heartbeat: minutes(3),
                twap_window: minutes(30),
                new_data_timeout: minutes(2),
            });
        }
        Ok(())
    }
}