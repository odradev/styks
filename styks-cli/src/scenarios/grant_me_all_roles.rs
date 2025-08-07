use odra::host::HostEnv;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    ContractProvider, DeployedContractsContainer,
};
use styks_contracts::styks_price_feed::{StyksPriceFeedRole, StyksPriceFeed};

pub struct GrantMeAllRoles;

impl ScenarioMetadata for GrantMeAllRoles {
    const NAME: &'static str = "GrantMeAllRoles";
    const DESCRIPTION: &'static str = "Grants all roles to the caller for testing purposes.";
}

impl Scenario for GrantMeAllRoles {
    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> core::result::Result<(), Error> {
        let mut contract = container.contract_ref::<StyksPriceFeed>(&env)?;
        let address = env.caller();

        if contract.has_role(&StyksPriceFeedRole::ConfigManager.role_id(), &address) {
            odra_cli::log("Already is ConfigManager.");
        } else {
            odra_cli::log("Granting ConfigManager role.");
            env.set_gas(2_500_000_000);
            contract.grant_role(&StyksPriceFeedRole::ConfigManager.role_id(), &address);
        }

        if contract.has_role(&StyksPriceFeedRole::PriceSupplier.role_id(), &address) {
            odra_cli::log("Already is PriceSupplier.");
        } else {
            odra_cli::log("Granting PriceSupplier role.");
            env.set_gas(2_500_000_000);
            contract.grant_role(&StyksPriceFeedRole::PriceSupplier.role_id(), &address);
        }

        Ok(())
    }
}
