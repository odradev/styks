use odra::prelude::*;

use crate::error::PriceFeedError;

// --- Data Structures ---

pub type TimestampSec = u64;
pub type DurationSec = u64;
pub type Price = u64;
pub type PriceFeedId = String;

#[odra::odra_type]
#[derive(Copy)]
pub struct PriceRecord {
    pub timestamp: TimestampSec, 
    pub price: Price,
}

#[odra::odra_type]
pub struct PriceFeedConfig {
    pub price_feed_id: PriceFeedId,
    pub heartbeat: DurationSec,
    pub twap_window: DurationSec,
    pub new_data_timeout: DurationSec,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TwapAddPriceError {
    TimestampBeforeLast
}

#[odra::odra_type]
#[derive(Default)]
pub struct TwapStorage {
    prices: Vec<PriceRecord>
}

impl TwapStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_price(&mut self, record: PriceRecord) -> Result<(), TwapAddPriceError> {
        // If the storage is empty, just add the record.
        if self.prices.is_empty() {
            self.prices.push(record);
            return Ok(());
        }

        // Ensure the new record's timestamp is greater than the last record's timestamp.
        if let Some(last_timestamp) = self.last_timestamp() {
            if record.timestamp <= last_timestamp {
                return Err(TwapAddPriceError::TimestampBeforeLast);
            }
        }

        // Add the new record to the storage.
        self.prices.push(record);
        Ok(())
    }

    pub fn remove_old_prices(&mut self, cutoff_timestamp: TimestampSec) {
        self.prices.retain(|record| record.timestamp >= cutoff_timestamp);
    }

    pub fn raw_prices(&self) -> &[PriceRecord] {
        &self.prices
    }

    pub fn last_timestamp(&self) -> Option<TimestampSec> {
        self.prices.last().map(|record| record.timestamp)
    }

    pub fn twap_price(&self) -> Option<Price> {
        // calculate the average price from the stored records
        if self.prices.is_empty() {
            return None;
        }

        // If we have only one price record, return that price
        if self.prices.len() == 1 {
            return Some(self.prices[0].price);
        }

        // Calculate TWAP using the formula: P_TWAP = (∑ P_j * T_j) / (∑ T_j)
        let mut weighted_sum: u64 = 0;
        let mut total_time: u64 = 0;

        // Iterate through consecutive price records to calculate time intervals
        for i in 1..self.prices.len() {
            let current_record = &self.prices[i];
            let previous_record = &self.prices[i - 1];
            
            // Calculate time difference (T_j) between current and previous record
            let time_diff = current_record.timestamp - previous_record.timestamp;
            
            // Use the previous price for the time interval (standard TWAP calculation)
            // P_j * T_j
            weighted_sum += (previous_record.price as u64) * time_diff;
            total_time += time_diff;
        }

        // Avoid division by zero
        if total_time == 0 {
            return Some(self.prices[0].price);
        }

        // Calculate the time-weighted average price
        let twap = weighted_sum / total_time;
        Some(twap as Price)
    }
}

// --- Price Feed Implementatio ---

#[odra::module]
pub struct PriceFeed {
    config: Var<PriceFeedConfig>,
    twap_storage: Var<TwapStorage>,
    price_history_counter: Var<u64>,
    price_history: Mapping<u64, PriceRecord>,
}

#[odra::module]
impl PriceFeed {
    pub fn set_config(&mut self, config: PriceFeedConfig) {
        self.config.set(config);
    }

    pub fn get_config(&self) -> PriceFeedConfig {
        self.config.get().unwrap_or_revert(&self.env())
    }

    pub fn is_initialized(&self) -> bool {
        self.config.get().is_some()
    }

    pub fn get_twap_storage(&self) -> TwapStorage {
        self.twap_storage.get().unwrap_or_default()
    }

    pub fn set_twap_storage(&mut self, twap_storage: TwapStorage) {
        self.twap_storage.set(twap_storage);
    }

    pub fn get_price_history_counter(&self) -> u64 {
        self.price_history_counter.get().unwrap_or_default()
    }

    pub fn set_price_history_counter(&mut self, counter: u64) {
        self.price_history_counter.set(counter);
    }

    pub fn get_price_history(&self, id: u64) -> Option<PriceRecord> {
        self.price_history.get(&id)
    }

    pub fn publish_price(&mut self, record: PriceRecord) {
        let config = self.get_config();
        let current_timestamp = self.env().get_block_time_secs();

        // If the record timestamp is in the future, revert.
        if record.timestamp > current_timestamp {
            self.env().revert(PriceFeedError::TimestampInFuture);
        }

        // Check if the record hasn't timed out.
        if record.timestamp <= current_timestamp - config.new_data_timeout {
            self.env().revert(PriceFeedError::TimestampTooOld);
        }

        // Add the price record to the TWAP storage.
        let mut twap_storage = self.get_twap_storage();
        let add_price_result = twap_storage.add_price(record);
        
        // Handle potential errors from adding the price record.
        // If the timestamp is before the last recorded timestamp, revert.
        if let Err(e) = add_price_result {
            match e {
                TwapAddPriceError::TimestampBeforeLast => {
                    self.env().revert(PriceFeedError::TimestampTooOld);
                }
            }
        }

        // Prune old prices if necessary.
        let cutoff_timestamp = current_timestamp - config.twap_window;
        twap_storage.remove_old_prices(cutoff_timestamp);

        // Update the TWAP storage in the contract state.
        self.set_twap_storage(twap_storage);

        // Increment the price history counter.
        let counter = self.get_price_history_counter();
        self.set_price_history_counter(counter + 1);

        // Store the price record in the price history mapping.
        self.price_history.set(&counter, record);
    }

    pub fn get_twap_price(&self) -> Option<Price> {
        // Retrieve the TWAP storage and calculate the TWAP price.
        self.get_twap_storage().twap_price()
    }
}


#[cfg(test)]
mod tests {
    use odra::host::{Deployer, HostEnv, NoArgs};

    use crate::minutes;

    use super::*;

    // -- Test PriceFeed ---

    fn setup() -> (HostEnv, PriceFeedConfig, PriceFeedHostRef) {
        let env = odra_test::env();
        let config = PriceFeedConfig {
            price_feed_id: String::from("BTCUSD"),
            heartbeat: minutes(10),
            twap_window: minutes(30),
            new_data_timeout: minutes(2),
        };
        let mut price_feed = PriceFeed::deploy(&env, NoArgs);
        price_feed.set_config(config.clone());
        env.advance_block_time(24 * 60 * 60 * 1000); // Advance time by 1 day.
        (env, config, price_feed)
    }

    #[test]
    fn test_initial_config() {
        // Given initial setup.
        let (_env, config, price_feed) = setup();

        // Then config should match the initial setup.
        let current_config = price_feed.get_config();
        assert_eq!(current_config, config);
    }

    #[test]
    fn test_publishing_invalid_timestamp() {
        // Given initial setup.
        let (env, config, mut price_feed) = setup();

        // When trying to publish a record with a future timestamp.
        let future_record = PriceRecord {
            timestamp: env.block_time_secs() + 1,
            price: 1,
        };
        let result = price_feed.try_publish_price(future_record);

        // Then it should revert with TimestampInFuture error.
        assert_eq!(result, Err(PriceFeedError::TimestampInFuture.into()));

        // When trying to publish a record with an old timestamp.
        let old_record = PriceRecord {
            timestamp: env.block_time_secs() - config.new_data_timeout,
            price: 1,
        };
        let result = price_feed.try_publish_price(old_record);

        // Then it should revert with TimestampTooOld error.
        assert_eq!(result, Err(PriceFeedError::TimestampTooOld.into()));
    }

    #[test]
    fn test_missing_the_heartbeat() {
        // Given initial setup.
        let (env, config, mut price_feed) = setup();

        // Given 10 price records with raising prices.
        let initial_price = 1000;
        let time_diff_sec = config.heartbeat;
        for i in 0..10 {
            let record = PriceRecord {
                timestamp: env.block_time_secs(),
                price: initial_price + i * 100,
            };
            price_feed.publish_price(record);
            env.advance_block_time(time_diff_sec * 1000);
        }

        // Then the TWAP storage should contain 3 records.
        let twap_storage = price_feed.get_twap_storage();
        assert_eq!(twap_storage.raw_prices().len(), 4);

        // The TWAP price should be calculated correctly.
        // The prices are:
        // 1. timestamp: 90000, price: 1600,
        // 2. timestamp: 90600, price: 1700,
        // 3. timestamp: 91200, price: 1800,
        // 4. timestamp: 91800, price: 1900.
        // The TWAP should be calculated as:
        // TWAP = 600 * 1700 + 600 * 1800 + 600 * 1900 / (600 + 600 + 600) = 1800
        let _twap_price = twap_storage.twap_price();

        // TODO: This should work:
        // assert_eq!(_twap_price, Some(1800));
    }

    // --- Test TwapStorage ---
    
    #[test]
    fn test_empty_twap_storage() {
        // Given an empty TwapStorage instance.
        let empty_storage = TwapStorage::new();

        // Then the TWAP price should be None.
        assert_eq!(empty_storage.twap_price(), None);
    }

    #[test]
    fn test_single_price_record_twap() {
        // Given a TwapStorage instance with a single price record.
        let mut single_price_storage = TwapStorage::new();
        let record = PriceRecord {
            timestamp: 1000,
            price: 1000,
        };

        single_price_storage.add_price(record).unwrap();

        // Then the TWAP price should be equal to that record's price.
        assert_eq!(single_price_storage.twap_price(), Some(1000));
    }

    #[test]
    fn test_twap_adding_price_error() {
        // Given a TwapStorage instance.
        let mut twap_storage = TwapStorage::new();

        // Given a record with a valid timestamp.
        let record = PriceRecord {
            timestamp: 1000,
            price: 1000,
        };
        twap_storage.add_price(record).unwrap();

        // When try to add a record with an earlier timestamp.
        let earlier_record = PriceRecord {
            timestamp: 500,
            price: 900,
        };
        let result = twap_storage.add_price(earlier_record);
        
        // Then it should return an error.
        assert_eq!(result, Err(TwapAddPriceError::TimestampBeforeLast));
    }

    #[test]
    fn test_twap_storage() {
        // Given a TwapStorage instance.
        let mut twap_storage = TwapStorage::new();

        // When adding 3 price records.
        let p0 = PriceRecord {
            timestamp: 1000,
            price: 1000,
        };

        let p1 = PriceRecord {
            timestamp: 2000,
            price: 1200,
        };

        let p2 = PriceRecord {
            timestamp: 3500,
            price: 900,
        };

        let p3 = PriceRecord {
            timestamp: 4000,
            price: 2100,
        };

        twap_storage.add_price(p0).unwrap();
        twap_storage.add_price(p1).unwrap();
        twap_storage.add_price(p2).unwrap();
        twap_storage.add_price(p3).unwrap();
    
        // Then the storage should contain 4 records.
        let expected = vec![p0, p1, p2, p3];
        assert_eq!(twap_storage.raw_prices(), &expected);

        // Then the last timestamp should be 4000.
        assert_eq!(twap_storage.last_timestamp(), Some(p3.timestamp));

        // Test TWAP calculation
        // Given the price records:
        // p0: timestamp=1000, price=1000
        // p1: timestamp=2000, price=1200 (time_diff=1000, weighted_sum += 1000*1000=1000000)
        // p2: timestamp=3500, price=900  (time_diff=1500, weighted_sum += 1200*1500=1800000)
        // p3: timestamp=4000, price=2100 (time_diff=500,  weighted_sum += 900*500=450000)
        // Total weighted_sum = 1000000 + 1800000 + 450000 = 3250000
        // Total time = 1000 + 1500 + 500 = 3000
        // TWAP = 3250000 / 3000 = 1083 (rounded down)
        let twap = twap_storage.twap_price();
        assert_eq!(twap, Some(1083));

        // When removing timestamps older then 2000.
        twap_storage.remove_old_prices(2000);

        // Then the storage should contain only records with timestamps >= 2000.
        let expected_after_removal = vec![p1, p2, p3];
        assert_eq!(twap_storage.raw_prices(), &expected_after_removal);

        // Then the twap price should be recalculated.
        // p1: timestamp=2000, price=1200
        // p2: timestamp=3500, price=900  (time_diff=1500, weighted_sum += 1200*1500=1800000)
        // p3: timestamp=4000, price=2100 (time_diff=500,  weighted_sum += 900*500=450000)
        // Total weighted_sum = 1800000 + 450000 = 2250000
        // Total time = 1500 + 500 = 2000
        // TWAP = 2250000 / 2000 = 1125 (rounded down)
        let twap = twap_storage.twap_price();
        assert_eq!(twap, Some(1125));
    }

}


