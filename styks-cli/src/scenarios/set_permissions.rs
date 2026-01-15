use std::fmt::Debug;

use odra::host::{HostEnv, HostRef};
use odra::prelude::*;
use odra_cli::{
    cspr,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    ContractProvider, DeployedContractsContainer,
};
use styks_contracts::make_supplier::StyksMakeSupplier;
use styks_contracts::styks_price_feed::{StyksPriceFeed, StyksPriceFeedRole};
use styks_contracts::supplier_role::SupplierRole;

pub struct SetPermissions;

impl ScenarioMetadata for SetPermissions {
    const NAME: &'static str = "SetPermissions";
    const DESCRIPTION: &'static str = "Setup testnet permissions.";
}

impl Scenario for SetPermissions {
    fn args(&self) -> Vec<odra_cli::CommandArg> {
        // vec![odra_cli::CommandArg::new(
        //     "address",
        //     "The address of a new PriceSupplier.",
        //     odra::schema::casper_contract_schema::NamedCLType::Key,
        // )]
        vec![]
    }
    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> core::result::Result<(), Error> {
        // let address = args.get_single::<Address>("address")?;
        let address =
            "account-hash-915691433d2c86c6145e46e3c5f3d266d87be6448de5dc8a4c4e710384372916";
        let address = Address::new(address).unwrap();
        odra_cli::log(format!("Setting permissions for address: {:?}", address));

        let feed = container.contract_ref::<StyksPriceFeed>(&env)?;
        let supplier = container.contract_ref::<StyksMakeSupplier>(&env)?;
        let deployer = env.caller();

        // Grant all Config roles to the deployer.
        odra_cli::log(format!("Setting permissions for address: {:?}", deployer));
        set_role(&feed, StyksPriceFeedRole::ConfigManager, deployer, env)?;
        set_role(&supplier, SupplierRole::ConfigManager, deployer, env)?;

        // Grant PriceSupplier and ConfigManager role to the account installed on the server.
        set_role(&feed, StyksPriceFeedRole::PriceSupplier, address, env)?;
        set_role(&supplier, SupplierRole::ConfigManager, address, env)?;

        // Grant PriceSupplier role to the StyksBlockySupplier in StyksPriceFeed.
        odra_cli::log("Setting permissions for StyksBlockySupplier contract.");
        set_role(&feed, StyksPriceFeedRole::PriceSupplier, supplier, env)?;


        Ok(())
    }
}

#[odra::external_contract]
trait RoleManager {
    fn has_role(&self, role: &[u8; 32], address: &Address) -> bool;
    fn grant_role(&mut self, role: &[u8; 32], address: &Address);
}

trait RoleId: Debug {
    fn get(&self) -> [u8; 32];
}

impl RoleId for StyksPriceFeedRole {
    fn get(&self) -> [u8; 32] {
        self.role_id()
    }
}

impl RoleId for SupplierRole {
    fn get(&self) -> [u8; 32] {
        self.role_id()
    }
}

fn set_role(
    contract_address: &dyn Addressable,
    role_id: impl RoleId,
    address: impl Addressable,
    env: &HostEnv,
) -> Result<(), Error> {
    let mut contract = RoleManagerHostRef::new(contract_address.address(), env.clone());
    if contract.has_role(&role_id.get(), &address.address()) {
        odra_cli::log(format!("Already has role: {:?}", role_id));
    } else {
        odra_cli::log(format!("Granting role: {:?}", role_id));
        env.set_gas(cspr!(2.5));
        contract.grant_role(&role_id.get(), &address.address());
    }
    Ok(())
}
