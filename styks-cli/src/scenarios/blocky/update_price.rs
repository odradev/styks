use core::panic;
use std::path::Path;

use odra::{casper_types::bytesrepr::Bytes, host::HostEnv};
use odra_cli::{
    cspr, scenario::{Args, Error, Scenario, ScenarioMetadata}, CommandArg, ContractProvider, DeployedContractsContainer
};
use styks_blocky_parser::{blocky_claims::BlockyClaims, blocky_output::BlockyOutput};
use styks_contracts::{styks_blocky_supplier::{StyksBlockySupplier, StyksBlockySupplierHostRef}, styks_price_feed::{StyksPriceFeed, StyksPriceFeedHostRef}};
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
        // updater.report_price();
        Ok(())
    }
}

pub struct Updater {
    env: HostEnv,
    feed_contract: StyksPriceFeedHostRef,
    supplier_contract: StyksBlockySupplierHostRef,
    coingecko_client: CoinGeckoClient,
    price_feed_id: String,
    use_blocky_supplier: bool,
}

impl Updater {
    pub fn new(env: HostEnv, container: &DeployedContractsContainer) -> Result<Self, Error> {
        let feed_contract = container.contract_ref::<StyksPriceFeed>(&env)?;
        let supplier_contract = container.contract_ref::<StyksBlockySupplier>(&env)?;
        let coingecko_client = CoinGeckoClient::new();
        Ok(Updater {
            env,
            feed_contract,
            supplier_contract,
            coingecko_client,
            price_feed_id: String::from("CSPRUSD"),
            use_blocky_supplier: true
        })
    }

    pub fn start(&mut self) {
        odra_cli::log("[x] Starting price update loop.");

        // Fetch the current configuration from the contract.        
        let config = self.feed_contract.get_config();
        odra_cli::log(format!("Current config: {:?}", config));

        loop {
            odra_cli::log("[x] Starting loop.");
            // Load last heartbeat time.
            let last_heartbeat = self.feed_contract.get_last_heartbeat().unwrap_or_default();
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
            // odra_cli::log(format!("Heartbeat status:\n{:#?}", heartbeat_status));
            odra_cli::log(format!(
                "Missed heartbeats since last heartbeat: {}",
                missed_heartbeat
            ));
            
            // If we're in the current heartbeat window, update the price.
            if let Some(current_window) = heartbeat_status.current {
                if current_window.middle == last_heartbeat {
                    odra_cli::log("Already updated price in this heartbeat window.");
                } else {
                    self.report_price();
                }    
            }

             // Load current time.
            let current_time = current_timestamp_secs();
            odra_cli::log(format!("Current time: {}", current_time));
            let next_heartbeat_time = heartbeat_status.next.middle;
            odra_cli::log(format!(
                "Next heartbeat time: {}",
                next_heartbeat_time
            ));
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

    pub fn report_price(&mut self) {
        if self.use_blocky_supplier {
            odra_cli::log("Reporting price via Blocky Supplier.");
            self.report_price_via_blocky_supplier();
        } else {
            odra_cli::log("Reporting price directly to the feed.");
            self.report_price_direct_to_feed();
        }
    }

    pub fn report_price_direct_to_feed(
        &mut self,
    ) {
        let current_time = current_timestamp_secs();
        let price = self.get_realtime_price();
        odra_cli::log(format!(
            "Updating price feed {} with price: ${} and timestamp: {}.",
            self.price_feed_id, price, current_time
        ));
        // Send price record to the contract.
        self.env.set_gas(cspr!(2.5));
        let result = self.feed_contract.try_add_to_feed(vec![(self.price_feed_id.clone(), price)]);
        match result {
            Ok(_) => odra_cli::log("Price updated successfully."),
            Err(e) => odra_cli::log(format!("Failed to update price: {:?}.", e)),
        }
    }

    pub fn report_price_via_blocky_supplier(&mut self) {
        // Call `make run-no-build` in the blocky-guest directory.
        odra_cli::log("Calling Blocky service to report price.");
        self.make_blocky_call();
        let output = self.read_blocky_output();
        let ta = output.ta();
        let signature = ta.signature_bytes();
        let data = ta.data();
        let claims = BlockyClaims::decode_fn_call_claims(&data).unwrap();
        let output_value = claims.output().unwrap();
        let price = output_value.price;
        let timestamp = output_value.timestamp;
        odra_cli::log(format!(
            "Updating price feed {} with price: ${} and timestamp: {}.",
            self.price_feed_id, price, timestamp
        ));

        self.env.set_gas(cspr!(4.0));
        let result = self.supplier_contract.try_report_signed_prices(
            Bytes::from(signature),
            Bytes::from(data),
        );
        match result {
            Ok(_) => odra_cli::log("Price updated successfully."),
            Err(e) => odra_cli::log(format!("Failed to update price: {:?}.", e)),
        }
    }

    pub fn make_blocky_call(&self) {
        let output = std::process::Command::new("make")
            .arg("run-no-build")
            .current_dir("blocky-guest")
            .output()
            .unwrap();

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            panic!("Failed to call Blocky service: {}", error_message);
        }
    }

    pub fn read_blocky_output(&self) -> BlockyOutput {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(manifest_dir).join("../blocky-guest/tmp/out.json");
        BlockyOutput::try_from_file(path).unwrap()
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
