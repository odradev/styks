use odra::{host::HostEnv, schema::casper_contract_schema::NamedCLType};
use odra_cli::{scenario::{Args, Error, Scenario, ScenarioMetadata}, CommandArg, ContractProvider, DeployedContractsContainer};
use styks_contracts::{price_feed_manager::PriceFeedManager};

pub struct UpdatePrice;
impl ScenarioMetadata for UpdatePrice {
    const NAME: &'static str = "UpdatePrice";
    const DESCRIPTION: &'static str = "Updates the price in the PriceFeedManager contract.";
}

impl Scenario for UpdatePrice {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("price_feed_id", "The ID of the price feed to add.", NamedCLType::String)
            .required()
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args
    ) -> core::result::Result<(), Error> {
        // Load data.
        let price_feed_id: String = args.get_single("price_feed_id")?;
        let coingecko_client = CoinGeckoClient::new();
        let price_cg = coingecko_client.get_price(&price_feed_id).unwrap();
        let price = (price_cg * 100_000.0) as u64;
        let current_time = env.block_time_secs();
        odra_cli::log(format!("Updating price feed {} with price: ${} and timestamp: {}.", price_feed_id, price_cg, current_time));

        // Sent price record to the contract.
        let mut contract = container.contract_ref::<PriceFeedManager>(&env)?;
        env.set_gas(2_500_000_000);
        contract.publish_price(&price_feed_id, price, current_time);

        // odra_cli::log("Price updated successfully.");
        Ok(())
    }
}

// --- Coingecko clinet ---
pub struct CoinGeckoClient {
    api_key: String,
}

impl CoinGeckoClient {
    pub fn new() -> Self {
        // Read COINGECKO_PRO_API_KEY key from environment variable.
        let api_key = std::env::var("COINGECKO_PRO_API_KEY")
            .expect("COINGECKO_PRO_API_KEY environment variable not set");
        CoinGeckoClient { api_key }
    }

    pub fn get_price(&self, price_feed_id: &str) -> Result<f64, String> {
        let currency = match price_feed_id {
            "CSPRUSD" => "casper-network",
            "BTCUSD" => "bitcoin",
            _ => return Err(format!("Unsupported price feed ID: {}", price_feed_id)),
        };
        let url = format!("https://api.coingecko.com/api/v3/simple/price?vs_currencies=usd&ids={}", currency);
        let response = ureq::get(url)
            .header("x-cg-demo-api-key", &self.api_key)
            .call();
        match response {
            Ok(mut resp) => {
                let body = resp.body_mut().read_to_string().unwrap();
                let json: serde_json::Value = serde_json::from_str(&body)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))?;
                if let Some(price) = json[currency]["usd"].as_f64() {
                    return Ok(price);
                } else {
                    // odra_cli::log("Price not found in response.");
                    // odra_cli::log(format!("Response: {}", json));
                    return Err("Price not found in response".to_string());
                }
            },
            Err(e) => Err(format!("Failed to fetch price: {}", e)),
        }
    }
}
