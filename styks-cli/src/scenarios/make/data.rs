use odra::host::HostEnv;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    DeployedContractsContainer,
};
use styks_make_parser::{input::NoncePayload, output::SignedPriceData, Data};

pub fn get_price_data<T: Into<NoncePayload>>(
    price_feed_id: &str,
    nonce: T,
) -> Result<SignedPriceData, String> {
    let api_key = std::env::var("CSPR_CLOUD_AUTH_TOKEN")
        .expect("CSPR_CLOUD_AUTH_TOKEN environment variable not set");

    let response = ureq::post(format!(
        "https://attested-api.cspr.cloud/rates/{price_feed_id}/latest"
    ))
    .content_type("application/json")
    .header("Authorization", api_key)
    .send(serde_json::to_vec(&nonce.into()).unwrap());
    match response {
        Ok(mut resp) => {
            let body = resp
                .body_mut()
                .read_to_string()
                .map_err(|e| format!("Failed to read response body: {}", e))?;
            serde_json::from_str::<Data<SignedPriceData>>(&body)
                .map(|result| result.data)
                .map_err(|e| format!("Failed to parse JSON: {}", e))
        }
        Err(e) => Err(format!("Failed to fetch price: {}", e)),
    }
}

pub struct GetPriceData;
impl ScenarioMetadata for GetPriceData {
    const NAME: &'static str = "GetPriceData";
    const DESCRIPTION: &'static str = "Updates the price in the PriceFeedManager contract.";
}

impl Scenario for GetPriceData {
    fn run(
        &self,
        _env: &HostEnv,
        _container: &DeployedContractsContainer,
        _args: Args,
    ) -> Result<(), Error> {
        let data = get_price_data("1", 67).unwrap();
        odra_cli::log(format!("Price data: {:?}", data));
        Ok(())
    }
}
