use odra::host::HostEnv;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};
use styks_contracts::styks_price_feed::{StyksPriceFeed, StyksPriceFeedHostRef};
use styks_core::heartbeat::Heartbeat;


pub struct UpdatePrice;
impl ScenarioMetadata for UpdatePrice {
    const NAME: &'static str = "UpdatePrice";
    const DESCRIPTION: &'static str = "Updates the price in the PriceFeedManager contract.";
}

impl Scenario for UpdatePrice {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> core::result::Result<(), Error> {
        let mut updater = Updater::new(env.clone(), container)?;
        updater.start();

        // // Load data.
        // let price_feed_id = String::from("CSPRUSD");
        // let coingecko_client = CoinGeckoClient::new();
        // let price_cg = coingecko_client.get_price(&price_feed_id).unwrap();
        // let price = (price_cg * 100_000.0) as u64;
        // let current_time = env.block_time_secs();
        // odra_cli::log(format!(
        //     "Updating price feed {} with price: ${} and timestamp: {}.",
        //     price_feed_id, price_cg, current_time
        // ));

        // // Sent price record to the contract.
        // let mut contract = container.contract_ref::<StyksPriceFeed>(&env)?;
        // env.set_gas(2_500_000_000);
        // contract.add_to_feed(vec![(price_feed_id, price)]);

        // odra_cli::log("Price updated successfully.");
        Ok(())
    }
}

pub struct Updater {
    env: HostEnv,
    contract: StyksPriceFeedHostRef,
    coingecko_client: CoinGeckoClient,
    price_feed_id: String,
}

impl Updater {
    pub fn new(env: HostEnv, container: &DeployedContractsContainer) -> Result<Self, Error> {
        let contract = container.contract_ref::<StyksPriceFeed>(&env)?;
        let coingecko_client = CoinGeckoClient::new();
        Ok(Updater {
            env,
            contract,
            coingecko_client,
            price_feed_id: String::from("CSPRUSD"),
        })
    }

    pub fn start(&mut self) {
        odra_cli::log("[x] Starting price update loop.");

        // Fetch the current configuration from the contract.        
        let config = self.contract.get_config();
        odra_cli::log(format!("Current config: {:?}", config));

        loop {
            odra_cli::log("[x] Starting loop.");
            // Load last heartbeat time.
            let last_heartbeat = self.contract.get_last_heartbeat().unwrap_or_default();
            odra_cli::log(format!("Last heartbeat time: {:?}", last_heartbeat));

            // Load current time.
            let current_time = current_timestamp_secs();
            odra_cli::log(format!("Current time:        {}", current_time));
            
            // Load Heartbeat state.
            let heartbeat = Heartbeat::new(
                current_time,
                config.heartbeat_interval,
                config.heartbeat_tolerance,
            ).unwrap();
            let heartbeat_status = heartbeat.current_state();
            let missed_heartbeat = heartbeat.count_missed_heartbeats_since(last_heartbeat);
            odra_cli::log(format!("Heartbeat status:\n{:#?}", heartbeat_status));
            odra_cli::log(format!(
                "Missed heartbeats since last heartbeat: {}",
                missed_heartbeat
            ));
            
            // If we're in the current heartbeat window, update the price.
            if let Some(current_window) = heartbeat_status.current {
                if current_window.middle == last_heartbeat {
                    odra_cli::log("Already updated price in this heartbeat window.");
                } else {
                    let price = self.get_realtime_price();
                    odra_cli::log(format!(
                        "Updating price feed {} with price: ${} and timestamp: {}.",
                        self.price_feed_id, price, current_time
                    ));
                    // Send price record to the contract.
                    self.env.set_gas(2_500_000_000);
                    let result = self.contract.try_add_to_feed(vec![(self.price_feed_id.clone(), price)]);
                    match result {
                        Ok(_) => odra_cli::log("Price updated successfully."),
                        Err(e) => odra_cli::log(format!("Failed to update price: {:?}.", e)),
                    }
                }    
            }

            let next_heartbeat_time = heartbeat_status.next.middle;
            let sleep_time = next_heartbeat_time.saturating_sub(current_time);
            odra_cli::log(format!(
                "Sleeping for {} seconds until next heartbeat.",
                sleep_time
            ));
            std::thread::sleep(std::time::Duration::from_secs(sleep_time));
            odra_cli::log("[x] Loop iteration completed.");
            odra_cli::log("--------------------------------------------------");
        }
    }

    pub fn get_realtime_price(&self) -> u64 {
        let price_cg = self.coingecko_client.get_price(&self.price_feed_id).unwrap();
        let price = (price_cg * 100_000.0) as u64;
        odra_cli::log(format!(
            "Current price for {}: ${}",
            self.price_feed_id, price_cg
        ));
        price
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
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?vs_currencies=usd&ids={}",
            currency
        );
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
            }
            Err(e) => Err(format!("Failed to fetch price: {}", e)),
        }
    }
}

fn current_timestamp_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let start = SystemTime::now();
    let timestamp = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    timestamp
}
