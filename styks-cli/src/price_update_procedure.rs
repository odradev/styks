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

fn next_heartbeat_time(
    current_time_sec: u64,
    period_sec: u64
) -> u64 {
    if current_time_sec % period_sec == 0 {
        current_time_sec
    } else {
        current_time_sec + (period_sec - (current_time_sec % period_sec))
    }
}

fn previous_heartbeat_time(
    current_time_sec: u64,
    period_sec: u64
) -> Option<u64> {
    if current_time_sec < period_sec {
        return None;
    }
    Some(next_heartbeat_time(current_time_sec, period_sec) - period_sec)
}

fn should_update_price(
    current_time: u64,
    last_update_time: u64,
    heartbeat: u64,
    
) -> bool {
    let prev_heartbeat_time = previous_heartbeat_time(current_time, heartbeat);
    match prev_heartbeat_time {
        Some(prev_time) => {
            // If the last update time is before the previous heartbeat time, we should update.
            last_update_time < prev_time
        }
        None => {
            // If there is no previous heartbeat time, we should update.
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_heartbeat_time() {
        assert_eq!(next_heartbeat_time(1000, 100), 1000);
        assert_eq!(next_heartbeat_time(1001, 100), 1100);
        assert_eq!(next_heartbeat_time(1050, 200), 1200);
    }

    #[test]
    fn test_previous_heartbeat_time() {
        assert_eq!(previous_heartbeat_time(1000, 1001), None);
        assert_eq!(previous_heartbeat_time(1000, 1000), Some(0));
        assert_eq!(previous_heartbeat_time(1000, 100), Some(900));
        assert_eq!(previous_heartbeat_time(1001, 100), Some(1000));
    }

    #[test]
    fn test_should_update_price() {
        // This is the heartbeat interval in seconds.
        let heartbeat = 100;
        
        // Test halfway through heartbeat.
        let current_time = 1000;
        let last_update_time = 950;
        assert!(!should_update_price(current_time, last_update_time, heartbeat));

        // Test just before heartbeat.
        let current_time = 1000;
        let last_update_time = 999;
        assert!(!should_update_price(current_time, last_update_time, heartbeat));

        // Test at the edge of heartbeat.
        let last_update_time = 900;
        let current_time = 1000;
        assert!(should_update_price(current_time, last_update_time, heartbeat));
        

    }
}