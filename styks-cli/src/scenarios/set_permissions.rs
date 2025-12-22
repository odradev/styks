use odra::prelude::*;
use odra::host::HostEnv;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    cspr, scenario::{Args, Error, Scenario, ScenarioMetadata}, CommandArg, ContractProvider, DeployedContractsContainer
};
use styks_contracts::{styks_blocky_supplier::{StyksBlockySupplerRole, StyksBlockySupplier, StyksBlockySupplierHostRef}, styks_price_feed::{StyksPriceFeed, StyksPriceFeedHostRef, StyksPriceFeedRole}};

pub struct SetPermissions;

impl ScenarioMetadata for SetPermissions {
    const NAME: &'static str = "SetPermissions";
    const DESCRIPTION: &'static str = "Setup testnet permissions.";
}

impl Scenario for SetPermissions {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new(
                "guardian-address",
                "The account hash for the Guardian role (emergency operations)",
                NamedCLType::String,
            )
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> core::result::Result<(), Error> {
        let mut feed = container.contract_ref::<StyksPriceFeed>(&env)?;
        let mut supplier = container.contract_ref::<StyksBlockySupplier>(&env)?;
        let deployer = env.caller();

        // Grant all Config roles to the deployer.
        odra_cli::log(format!("Setting permissions for address: {:?}", deployer));
        set_role_feed(&mut feed, &StyksPriceFeedRole::ConfigManager, &deployer, env)?;
        set_role_supplier(&mut supplier, &StyksBlockySupplerRole::ConfigManager, &deployer, env)?;

        // Grant PriceSupplier role to the account installed on the server.
        let address = "account-hash-915691433d2c86c6145e46e3c5f3d266d87be6448de5dc8a4c4e710384372916";
        let address = Address::new(address).unwrap();
        odra_cli::log(format!("Setting permissions for address: {:?}", address));
        set_role_feed(&mut feed, &StyksPriceFeedRole::PriceSupplier, &address, env)?;

        // Grant PriceSupplier role to the StyksBlockySupplier in StyksPriceFeed.
        odra_cli::log("Setting permissions for StyksBlockySupplier contract.");
        set_role_feed(
            &mut feed,
            &StyksPriceFeedRole::PriceSupplier,
            &supplier.address(),
            env,
        )?;

        // Grant Guardian role to separate address (if provided)
        if let Ok(guardian_addr) = args.get_single::<String>("guardian-address") {
            // Leak the string for 'static lifetime (acceptable for CLI code)
            let guardian_addr: &'static str = Box::leak(guardian_addr.into_boxed_str());
            let guardian = Address::new(guardian_addr)
                .expect("Invalid guardian address format");
            odra_cli::log(format!("Setting Guardian permissions for address: {:?}", guardian));
            set_role_supplier(&mut supplier, &StyksBlockySupplerRole::Guardian, &guardian, env)?;
        } else {
            odra_cli::log("Warning: No guardian-address provided. Guardian role not granted.");
        }

        Ok(())
    }
}

fn set_role_feed(
    contract: &mut StyksPriceFeedHostRef,
    role: &StyksPriceFeedRole,
    address: &Address,
    env: &HostEnv,
) -> Result<(), Error> {

    if contract.has_role(&role.role_id(), address) {
        odra_cli::log(format!("Already has role: {:?} in StyksPriceFeed", role));
    } else {
        odra_cli::log(format!("Granting role: {:?} in StyksPriceFeed", role));
        env.set_gas(cspr!(2.5));
        contract.grant_role(&role.role_id(), address);
    }
    Ok(())
}

fn set_role_supplier(
    contract: &mut StyksBlockySupplierHostRef,
    role: &StyksBlockySupplerRole,
    address: &Address,
    env: &HostEnv,
) -> Result<(), Error> {

    if contract.has_role(&role.role_id(), address) {
        odra_cli::log(format!("Already has role: {:?} in StyksBlockySupplier", role));
    } else {
        odra_cli::log(format!("Granting role: {:?} in StyksBlockySupplier", role));
        env.set_gas(cspr!(2.5));
        contract.grant_role(&role.role_id(), address);
    }
    Ok(())
}