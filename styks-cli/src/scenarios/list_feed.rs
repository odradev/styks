use odra::{host::HostEnv, schema::casper_contract_schema::NamedCLType};
use odra_cli::{scenario::{Args, Error, Scenario, ScenarioMetadata}, CommandArg, ContractProvider, DeployedContractsContainer};
use styks_contracts::price_feed_manager::{PriceFeedManager};

pub struct ListFeed;

impl ScenarioMetadata for ListFeed {
    const NAME: &'static str = "ListFeed";
    const DESCRIPTION: &'static str = "Adds a CSPR feed to the PriceFeedManager contract.";
}

impl Scenario for ListFeed {
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
        let contract = container.contract_ref::<PriceFeedManager>(&env)?;
        
        let price_feed_id: String = args.get_single("price_feed_id")?;
        let price = contract.get_price(&price_feed_id);
        if let Some(price) = price {
            odra_cli::log(format!("Price feed {} has price: ${}", price_feed_id, parse_price(price)));
        } else {
            odra_cli::log(format!("Price feed {} is not initialized.", price_feed_id));
            return Err(Error::OdraError { message: format!("Price feed {} is not initialized.", price_feed_id) });
        };

        let history_records_count = contract.get_price_feed_history_counter(&price_feed_id);
        odra_cli::log(format!("Price feed {} has {} history records.", price_feed_id, history_records_count));

        let current_time = env.block_time_secs();
        odra_cli::log(format!("Current timestamp: {}", current_time));

        if history_records_count == 0 {
            odra_cli::log("No history records found.");
            return Ok(());
        }

        let to_print = if history_records_count < 5 {
            history_records_count
        } else {
            5
        };

        odra_cli::log(format!("Last {to_print} history records:"));
        // Print last 5 history records.
        for i in 0..to_print {
            let record_id = history_records_count - i - 1;
            if let Some(record) = contract.get_price_history(&price_feed_id, record_id) {
                let price = parse_price(record.price);
                let duration = current_time - record.timestamp;
                let duration_str = parse_duration(duration);
                odra_cli::log(format!(
                    "[x] RecordId({}): Price: ${}, Timestamp: {} ({} ago).",
                    record_id, price, record.timestamp, duration_str
                ));
            } else {
                odra_cli::log(format!("Record {} not found.", record_id));
            }
        }
        Ok(())
    }
}

fn parse_price(price: u64) -> f64 {
    price as f64 / 100_000.0
}

// Returns a human-readable duration string:
// e.g. "1 hour", "2 days", "3 months", "45 seconds".
// or more complex like "1 year, 2 months, 3 days".
fn parse_duration(duration: u64) -> String {
    let mut parts = Vec::new();
    let seconds = duration % 60;
    let minutes = (duration / 60) % 60;
    let hours = (duration / 3600) % 24;
    let days = (duration / 86400) % 30;
    let months = (duration / 2592000) % 12;
    let years = duration / 31536000;

    if years > 0 {
        parts.push(format!("{} year{}", years, if years > 1 { "s" } else { "" }));
    }
    if months > 0 {
        parts.push(format!("{} month{}", months, if months > 1 { "s" } else { "" }));
    }
    if days > 0 {
        parts.push(format!("{} day{}", days, if days > 1 { "s" } else { "" }));
    }
    if hours > 0 {
        parts.push(format!("{} hour{}", hours, if hours > 1 { "s" } else { "" }));
    }
    if minutes > 0 {
        parts.push(format!("{} minute{}", minutes, if minutes > 1 { "s" } else { "" }));
    }
    if seconds > 0 {
        parts.push(format!("{} second{}", seconds, if seconds > 1 { "s" } else { "" }));
    }

    parts.join(", ")

}