use odra::host::HostEnv;
use odra_cli::{scenario::{Args, Error, Scenario, ScenarioMetadata}, ContractProvider, DeployedContractsContainer};
use styks_contracts::price_feed_manager::{ContractRole, PriceFeedManager};

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
        _args: Args
    ) -> core::result::Result<(), Error> {
        let mut contract = container.contract_ref::<PriceFeedManager>(&env)?;
        let address = env.caller();

        if contract.has_role(&ContractRole::PriceFeedManager.role_id(), &address) {
            odra_cli::log("Already has PriceFeedManager.");
        } else {
            odra_cli::log("Granting PriceFeedManager role.");
            env.set_gas(2_500_000_000);
            contract.grant_role(&ContractRole::PriceFeedManager.role_id(), &address);
        }

        if contract.has_role(&ContractRole::PriceSuppliers.role_id(), &address) {
            odra_cli::log("Already has PriceSuppliers.");
        } else {
            odra_cli::log("Granting PriceSuppliers role.");
            env.set_gas(2_500_000_000);
            contract.grant_role(&ContractRole::PriceSuppliers.role_id(), &address);
        }

        Ok(())
    }
}
