# Quick Start Guide

This guide walks you through setting up and using the Styks Measurement Anchored price oracle.

---

## Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- `cargo-odra` installed
- Access to a Casper node (testnet or mainnet)
- Blocky guest program built and deployed
- CoinGecko API key

---

## 1. Build Contracts

```bash
# Build all contracts to WASM
just build-contracts

# Output:
# wasm/StyksPriceFeed.wasm
# wasm/StyksBlockySupplier.wasm
```

---

## 2. Deploy Contracts

Deploy using the Odra CLI:

```bash
# Deploy StyksPriceFeed
just cli deploy StyksPriceFeed

# Deploy StyksBlockySupplier
just cli deploy StyksBlockySupplier
```

---

## 3. Configure Contracts

### Set Permissions

Grant roles to appropriate accounts:

```bash
just cli scenario SetPermissions
```

This grants:
- `ConfigManagerRole` to the deployer
- `PriceSupplierRole` to the supplier contract

### Set Configuration

Configure both contracts with correct parameters:

```bash
just cli scenario SetConfig
```

This configures:
- `StyksPriceFeed`: Heartbeat interval, TWAP window, price feed IDs
- `StyksBlockySupplier`: WASM hash, measurements, feed mappings, tolerances

---

## 4. Run Price Updater

Set your CoinGecko API key:

```bash
export COINGECKO_PRO_API_KEY="your-api-key"
```

Start the price update loop:

```bash
just cli scenario UpdatePrice
```

The updater will:
1. Register a signer on startup
2. Wait for heartbeat windows
3. Fetch prices from Blocky
4. Submit signed prices to the contract
5. Loop continuously

---

## 5. Query Prices

From your application, query the `StyksPriceFeed` contract:

```rust
use styks_contracts::styks_price_feed::StyksPriceFeedHostRef;

// Get TWAP price
let price = feed.get_twap_price("CSPRUSD".to_string());
println!("CSPR/USD: ${}", price as f64 / 100_000.0);

// Get last heartbeat time
let last_heartbeat = feed.get_last_heartbeat();
println!("Last update: {}", last_heartbeat);
```

---

## Configuration Reference

### StyksPriceFeedConfig

```rust
StyksPriceFeedConfig {
    heartbeat_interval: 30 * 60,  // 30 minutes between updates
    heartbeat_tolerance: 60,       // 1 minute tolerance window
    twap_window: 3,                // Average over 3 heartbeats
    twap_tolerance: 1,             // Allow 1 missed heartbeat
    price_feed_ids: vec!["CSPRUSD".to_string()],
}
```

### StyksBlockySupplierConfig

```rust
StyksBlockySupplierConfig {
    wasm_hash: "baadaf...".to_string(),
    expected_function: "priceFunc".to_string(),
    allowed_measurements: vec![MeasurementRule {
        platform: "nitro".to_string(),
        code: "pcr0.pcr1.pcr2".to_string(),
    }],
    coingecko_feed_ids: vec![
        ("Gate_CSPR_USD".to_string(), "CSPRUSD".to_string())
    ],
    price_feed_address: feed_address,
    timestamp_tolerance: 20 * 60,  // 20 minutes max age
    signer_ttl_secs: 24 * 60 * 60, // 24 hour signer cache
}
```

---

## Troubleshooting

### "ConfigNotSet" Error

The contract is not configured. Run:
```bash
just cli scenario SetConfig
```

### "MeasurementNotAllowed" Error

The enclave measurement does not match the allowlist. Either:
1. Update the allowlist with the correct measurement
2. Verify you're running the expected enclave build

### "SignerExpired" Error

The cached signer has exceeded its TTL. This is normal - the updater will automatically fall back to slow path and re-register.

### "TimestampTooOld" Error

The price data is older than `timestamp_tolerance`. Check:
1. Network connectivity to Blocky
2. System clock synchronization
3. Consider increasing `timestamp_tolerance`

### "ReplayAttack" Error

The same price data was submitted twice. This is a safety check - wait for a new price update from Blocky.

### "ContractPaused" Error

The contract has been paused by a Guardian. Contact the contract administrator.

---

## Updating Measurements

When the Blocky enclave is updated:

1. Get the new measurement from the enclave attestation
2. Update the configuration:

```rust
let new_measurement = MeasurementRule {
    platform: "nitro".to_string(),
    code: "new_pcr0.new_pcr1.new_pcr2".to_string(),
};

let mut config = supplier.get_config();
config.allowed_measurements.push(new_measurement);
supplier.set_config(config);
```

3. Optionally remove old measurements after transition period

---

## Emergency Procedures

### Pause the System

If you detect anomalous behavior:

```rust
supplier.pause();
```

### Revoke a Compromised Signer

If a signing key is compromised:

```rust
supplier.revoke_signer(compromised_signer_id);
```

### Resume Operations

After resolving the issue:

```rust
supplier.unpause();
```

---

## Integration Example

Complete example of querying prices from a smart contract:

```rust
use odra::prelude::*;
use styks_contracts::styks_price_feed::{StyksPriceFeed, StyksPriceFeedContractRef};

#[odra::module]
pub struct MyContract {
    price_feed: External<StyksPriceFeedContractRef>,
}

#[odra::module]
impl MyContract {
    pub fn init(&mut self, price_feed_address: Address) {
        self.price_feed.set(price_feed_address);
    }

    pub fn get_cspr_price(&self) -> u64 {
        let feed = self.price_feed.get().unwrap();
        feed.get_twap_price("CSPRUSD".to_string())
    }

    pub fn do_something_with_price(&self) {
        let price = self.get_cspr_price();
        // Price is in 5 decimal places (e.g., 516 = $0.00516)
        // Use for calculations...
    }
}
```

---

## Next Steps

- Read the [Architecture Documentation](./measurement-anchored-model.md)
- Review the [API Reference](./api-reference.md)
- Understand the [Security Model](./security-model.md)
