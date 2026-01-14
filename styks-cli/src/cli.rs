//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use odra::{
    contract_def::HasIdent,
    host::{HostEnv, InstallConfig, NoArgs},
};
use odra_cli::{cspr, deploy::DeployScript, DeployedContractsContainer, DeployerExt, OdraCli};
use styks_contracts::{
    make_supplier::StyksMakeSupplier, styks_blocky_supplier::StyksBlockySupplier,
    styks_price_feed::StyksPriceFeed,
};

mod scenarios;

pub struct ContractsDeployScript;
impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        let cfg = InstallConfig {
            package_named_key: StyksPriceFeed::ident(),
            is_upgradable: true,
            allow_key_override: true,
        };
        StyksPriceFeed::load_or_deploy_with_cfg(env, NoArgs, cfg, container, cspr!(400))?;

        let cfg = InstallConfig {
            package_named_key: StyksMakeSupplier::ident(),
            is_upgradable: true,
            allow_key_override: true,
        };
        StyksMakeSupplier::load_or_deploy_with_cfg(env, NoArgs, cfg, container, cspr!(600))?;
        Ok(())
    }
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("Styks CLI Tool")
        .deploy(ContractsDeployScript)
        .contract::<StyksPriceFeed>()
        .contract::<StyksBlockySupplier>()
        .contract::<StyksMakeSupplier>()
        .scenario(scenarios::SetPermissions)
        .scenario(scenarios::SetConfig)
        .scenario(scenarios::UpdatePrice)
        .scenario(scenarios::GetPriceData)
        .scenario(scenarios::ReportPriceDirectly)
        // .scenario(scenarios::ListFeed)
        .build()
        .run();
}
