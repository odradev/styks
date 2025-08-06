use std::time::Instant;

use styks_contracts::price_feed::{PriceFeedConfig, PriceFeedId, PriceRecord};


trait PriceUpdateProcedure {
    fn get_price_feed_id(&self) -> PriceFeedId;
    fn get_price_feed_config(&self, price_feed_id: &PriceFeedId) -> PriceFeedConfig;
    fn fetch_latest_price_record_from_feed(&self, price_feed_id: &PriceFeedId) -> Option<PriceRecord>;


    fn run(&mut self) {
        // Price Feed ID comes from argument.
        let price_feed_id = self.get_price_feed_id();

        // Price Feed Config is fetched from the contract.
        let price_feed_config = self.get_price_feed_config(&price_feed_id);

        // Load latest price from feed.
        let latest_price_record = self.fetch_latest_price_record_from_feed(&price_feed_id);

        // Exit if current time is before next expected heartbeat.
        if let Some(record) = latest_price_record {
            // let current_time = self.get_current_timestamp();
        }
    }
}
