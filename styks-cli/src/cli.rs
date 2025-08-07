//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use odra::{contract_def::HasIdent, host::{HostEnv, InstallConfig, NoArgs}};
use odra_cli::{deploy::DeployScript, DeployedContractsContainer, DeployerExt, OdraCli};
use styks_contracts::styks_price_feed::StyksPriceFeed;

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
        StyksPriceFeed::load_or_deploy_with_cfg(env, NoArgs, cfg, container, 400_000_000_000)?;
        Ok(())
    }
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("Styks CLI Tool")
        .deploy(ContractsDeployScript)
        .contract::<StyksPriceFeed>()
        // .contract::<Flapper>()
        .scenario(scenarios::GrantMeAllRoles)
        .scenario(scenarios::SetConfig)
        .scenario(scenarios::UpdatePrice)
        // .scenario(scenarios::ListFeed)
        .build()
        .run();
}
