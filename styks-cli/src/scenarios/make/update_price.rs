use odra::{
    casper_types::{bytesrepr::Bytes, PublicKey},
    host::HostEnv,
    prelude::OdraError,
};
use odra_cli::{
    cspr,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    ContractProvider, DeployedContractsContainer,
};
use serde_json::Value;
use styks_contracts::{
    make_supplier::{StyksMakeSupplier, StyksMakeSupplierHostRef},
    styks_price_feed::{StyksPriceFeed, StyksPriceFeedHostRef},
};
use styks_core::heartbeat::Heartbeat;
use styks_make_parser::output::SignedPriceData;

use crate::scenarios::make::data;

pub struct UpdatePrice;

impl ScenarioMetadata for UpdatePrice {
    const NAME: &'static str = "UpdatePrice";
    const DESCRIPTION: &'static str = "Updates the price in the PriceFeedManager contract.";
}

impl Scenario for UpdatePrice {
    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> core::result::Result<(), Error> {
        let mut updater = Updater::new(&env, container)?;
        updater.start()
    }
}

pub struct ReportPriceDirectly;

impl ScenarioMetadata for ReportPriceDirectly {
    const NAME: &'static str = "ReportPriceDirectly";
    const DESCRIPTION: &'static str = "Sends the price directly into the PriceFeedManager.";
}

impl Scenario for ReportPriceDirectly {
    fn args(&self) -> Vec<odra_cli::CommandArg> {
        vec![odra_cli::CommandArg::new(
            "feed_id",
            "Price feed id that the Make service expects.",
            odra::schema::casper_contract_schema::NamedCLType::String,
        )]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> core::result::Result<(), Error> {
        let mut feed_contract = container.contract_ref::<StyksPriceFeed>(&env)?;
        let feed_id = args.get_single::<String>("feed_id")?;

        let current_time = current_timestamp_secs();
        let price = self.get_realtime_price(&feed_id);

        odra_cli::log(format!(
            "Updating price feed {} with price: ${} and timestamp: {}.",
            feed_id, price, current_time
        ));
        // Send price record to the contract.
        env.set_gas(cspr!(2.5));
        let result = feed_contract.try_add_to_feed(vec![(feed_id, price)]);
        match result {
            Ok(_) => odra_cli::log("Price updated successfully."),
            Err(e) => odra_cli::log(format!("Failed to update price: {:?}.", e)),
        }
        Ok(())
    }
}

impl ReportPriceDirectly {
    fn get_realtime_price(&self, feed_id: &str) -> u64 {
        let signed_data = super::data::get_price_data(feed_id, current_timestamp_secs())
            .expect("Couldn't get price data");
        let value = serde_json::from_str::<Value>(&signed_data.response_body)
            .expect("Should be a valid json");
        let price = value["data"]["amount"]
            .as_f64()
            .expect("Should be a f64 value");
        odra_cli::log(format!("Current price for {}: ${}", feed_id, price));
        (price * 100_000_000.0) as u64
    }
}

pub struct Updater<'a> {
    env: &'a HostEnv,
    feed_contract: StyksPriceFeedHostRef,
    supplier_contract: StyksMakeSupplierHostRef,
    price_feed_id: String,
}

impl<'a> Updater<'a> {
    pub fn new(env: &'a HostEnv, container: &DeployedContractsContainer) -> Result<Self, Error> {
        let feed_contract = container.contract_ref::<StyksPriceFeed>(env)?;
        let supplier_contract = container.contract_ref::<StyksMakeSupplier>(env)?;
        Ok(Updater {
            env,
            feed_contract,
            supplier_contract,
            price_feed_id: String::from("1"),
        })
    }
}

impl Updater<'_> {
    fn start(&mut self) -> Result<(), Error> {
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
            )
            .map_err(|e| Error::OdraError {
                message: format!("Heartbeat error: {:?}", e),
            })?;
            let heartbeat_status = heartbeat.current_state();
            let missed_heartbeat = heartbeat.count_missed_heartbeats_since(last_heartbeat);
            odra_cli::log(format!(
                "Missed heartbeats since last heartbeat: {}",
                missed_heartbeat
            ));

            // If we're in the current heartbeat window, update the price.
            if let Some(current_window) = heartbeat_status.current {
                if current_window.middle == last_heartbeat {
                    odra_cli::log("Already updated price in this heartbeat window.");
                } else {
                    self.report_price_via_supplier()?;
                }
            }

            // Load current time.
            let current_time = current_timestamp_secs();
            odra_cli::log(format!("Current time: {}", current_time));
            let next_heartbeat_time = heartbeat_status.next.middle;
            odra_cli::log(format!("Next heartbeat time: {}", next_heartbeat_time));
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

    fn report_price_via_supplier(&mut self) -> Result<(), Error> {
        odra_cli::log("Calling Make service to report price.");
        let signed_data = data::get_price_data(&self.price_feed_id, current_timestamp_secs())
            .expect("Failed to get price data from Make service.");

        let public_key = signed_data
            .public_key()
            .expect("Failed to get public key from signed data.");
        if &public_key != self.supplier_contract.get_config().public_key() {
            if self.update_public_key(public_key).is_err() {
                odra_cli::log("Failed to update public key in MakeSupplier contract.");
                return Err(Error::OdraError {
                    message: "Public key update error".into(),
                });
            }
        }

        let result = self.report_price(signed_data);
        match result {
            Ok(_) => odra_cli::log("Price updated successfully."),
            Err(e) => odra_cli::log(format!("Failed to update price: {:?}.", e)),
        }
        Ok(())
    }

    fn update_public_key(&mut self, new_key: PublicKey) -> Result<(), OdraError> {
        odra_cli::log("Updating public key in MakeSupplier contract.");
        self.env.set_gas(cspr!(3.0));
        self.supplier_contract.try_update_public_key(new_key)
    }

    fn report_price(&mut self, signed_price: SignedPriceData) -> Result<(), OdraError> {
        odra_cli::log("Reporting price via Make Supplier.");

        self.env.set_gas(cspr!(4.0));
        self.supplier_contract.try_report_signed_prices(
            Bytes::from(
                signed_price
                    .signature_bytes()
                    .expect("Failed to get signature bytes from signed data."),
            ),
            Bytes::from(signed_price.payload_bytes()),
        )
    }
}

fn current_timestamp_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let start = SystemTime::now();
    let timestamp = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    timestamp
}
