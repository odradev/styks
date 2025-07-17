//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use odra::host::{HostEnv, NoArgs};
use odra_cli::{
    deploy::DeployScript,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer, DeployerExt,
    OdraCli,
};

pub struct ContractsDeployScript;

impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        // let _ = Flipper::load_or_deploy(
        //     &env,
        //     NoArgs,
        //     container,
        //     250_000_000_000, // Adjust gas limit as needed
        // )?;

        // let _ = Flapper::load_or_deploy(
        //     &env,
        //     NoArgs,
        //     container,
        //     250_000_000_000, // Adjust gas limit as needed
        // )?;

        Ok(())
    }
}


/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for styks smart contract")
        .deploy(ContractsDeployScript)
        // .contract::<Flipper>()
        // .contract::<Flapper>()
        // .scenario(FlipThemAll)
        .build()
        .run();
}
