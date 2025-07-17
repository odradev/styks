# Styks

Styks is the first casper onchain oracle.
It's very first task is providing CSPRUSD price feed with given frequency.

## Styks Smart Contracts

## StyksFeedManager

- Roles:
    - `Admin`,
        - Manages roles of other accounts.
    - `PriceFeedManagers`
        - Can manage PriceFeeds and define their configurations.
    - `PriceSuppliers`
        - Can feed the PriceFeed with new data.
    - `Anyone`
        - Can read the twap price at any time (also the latest).
        - Can list all PriceFeeds and their configurations.

If `PriceSuppliers` missed the heartbeat, then they have to provide correct
price feeds for at least `TWAP_window_minutes` for the price to be valid.


`PriceFeedConfiguration`:
- `price_feed_id` - Unique identifier for the price feed ex. CSPRUSD.
- `hearthbeat_minutes` - How often the price feed should be updated.
- `twap_window_minutes` - Based on this window the TWAP price is calculated.

## How price and TWAP is calculated and stored onchain.
```rust
#[odra::odra_struct]
pub struct PriceEntry {
    timestamp: u64, // Timestamp of the price entry
    price: u128, // Price in smallest unit (e.g., wei for ETH)
}

#[odra::odra_struct]
pub struct PriceFeedConfiguration {
    price_feed_id: String,
    heartbeat_minutes: u64,
    twap_window_minutes: u64,
}

#[odra::module]
pub struct StyksFeed {
    config: Var<PriceFeedConfiguration>,
    current_data_id: Var<DataId>, // Unique identifier for the current data entry
    data: Mapping<DataId, PriceEntry>, // Maps data_id to PriceEntry
    latest_data_for_twap: Var<Vec<PriceEntry>>, // Stores entries for TWAP calculation
}

impl StyksFeed {
    pub fn update_price(&mut self, price: u128);
    pub fn get_latest_twap_price(&self) -> Option<u128>;
}

#[odra::module]
pub struct StyksFeedManager {
    access_control: AccessControl, // Manages roles and permissions
    price_feeds: Mapping<String, StyksFeed>, // Maps price_feed_id to Sty
}


```

## Ideas
- Emit CEP95 NFTs on interesting price movements.