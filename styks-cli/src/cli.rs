//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use odra::host::{HostEnv, NoArgs};
use odra_cli::{
    deploy::DeployScript, DeployedContractsContainer, DeployerExt,
    OdraCli,
};
use styks_contracts::price_feed_manager::{PriceFeedManager};

mod scenarios;

pub struct ContractsDeployScript;
impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        PriceFeedManager::load_or_deploy(
            env, NoArgs, container, 400_000_000_000)?;

        Ok(())
    }
}


/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("Styks CLI Tool")
        .deploy(ContractsDeployScript)
        .contract::<PriceFeedManager>()
        // .contract::<Flapper>()
        .scenario(scenarios::GrantMeAllRoles)
        .scenario(scenarios::AddFeed)
        .scenario(scenarios::UpdatePrice)
        .scenario(scenarios::ListFeed)
        .build()
        .run();
}
