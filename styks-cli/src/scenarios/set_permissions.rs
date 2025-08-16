use odra::prelude::*;
use odra::host::HostEnv;
use odra_cli::{
    cspr, scenario::{Args, Error, Scenario, ScenarioMetadata}, ContractProvider, DeployedContractsContainer
};
use styks_contracts::styks_price_feed::{StyksPriceFeed, StyksPriceFeedHostRef, StyksPriceFeedRole};

pub struct SetPermissions;

impl ScenarioMetadata for SetPermissions {
    const NAME: &'static str = "SetPermissions";
    const DESCRIPTION: &'static str = "Setup testnet permissions.";
}

impl Scenario for SetPermissions {
    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> core::result::Result<(), Error> {
        let mut contract = container.contract_ref::<StyksPriceFeed>(&env)?;
        let address = env.caller();

        odra_cli::log(format!("Setting permissions for address: {:?}", address));
        // Grant all roles to the caller.
        set_role(&mut contract, &StyksPriceFeedRole::ConfigManager, &address, env)?;
        set_role(&mut contract, &StyksPriceFeedRole::PriceSupplier, &address, env)?;

        // Grant PriceSupplier role to the account installed on the server.
        let address = "account-hash-915691433d2c86c6145e46e3c5f3d266d87be6448de5dc8a4c4e710384372916";
        let address = Address::new(address).unwrap();

        odra_cli::log(format!("Setting permissions for address: {:?}", address));
        set_role(&mut contract, &StyksPriceFeedRole::PriceSupplier, &address, env)?;
        Ok(())
    }
}

fn set_role(
    contract: &mut StyksPriceFeedHostRef,
    role: &StyksPriceFeedRole,
    address: &Address,
    env: &HostEnv,
) -> Result<(), Error> {

    if contract.has_role(&role.role_id(), address) {
        odra_cli::log(format!("Already has role: {:?}", role));
    } else {
        odra_cli::log(format!("Granting role: {:?}", role));
        env.set_gas(cspr!(2.5));
        contract.grant_role(&role.role_id(), address);
    }
    Ok(())
}