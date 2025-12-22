# API Reference

## StyksBlockySupplier Contract

The `StyksBlockySupplier` contract serves as a bridge between the Blocky attestation service and the `StyksPriceFeed` contract. It verifies that price data originates from a trusted enclave before forwarding to the price feed.

---

## Table of Contents

1. [Roles](#roles)
2. [Error Codes](#error-codes)
3. [Data Types](#data-types)
4. [Configuration Entrypoints](#configuration-entrypoints)
5. [Signer Management Entrypoints](#signer-management-entrypoints)
6. [Price Reporting Entrypoints](#price-reporting-entrypoints)
7. [Operational Control Entrypoints](#operational-control-entrypoints)
8. [Query Entrypoints](#query-entrypoints)

---

## Roles

### AdminRole

```rust
const ADMIN_ROLE: [u8; 32] = [1u8; 32];
```

Full administrative access:
- Configure contract settings
- Grant/revoke all roles
- Pause/unpause operations
- Revoke signers

### ConfigManagerRole

```rust
const CONFIG_MANAGER_ROLE: [u8; 32] = [2u8; 32];
```

Configuration management:
- Update contract configuration

### GuardianRole

```rust
const GUARDIAN_ROLE: [u8; 32] = [4u8; 32];
```

Operational controls:
- Pause/unpause contract
- Revoke signers

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 46000 | `NotConfigManager` | Caller lacks ConfigManager role |
| 46001 | `ConfigNotSet` | Contract not yet configured |
| 46002 | `NotPriceSupplier` | Caller lacks PriceSupplier role |
| 46003 | `HashMismatch` | WASM hash does not match configuration |
| 46004 | `PriceFeedIdMismatch` | Price feed ID not in allowed list |
| 46005 | `InvalidSignature` | ECDSA signature verification failed |
| 46006 | `TimestampTooOld` | Price timestamp exceeds tolerance |
| 46400 | `ContractPaused` | Contract is paused |
| 46401 | `SignerNotFound` | Signer ID not in cache |
| 46402 | `SignerRevoked` | Signer has been revoked |
| 46403 | `SignerExpired` | Signer TTL exceeded |
| 46404 | `MeasurementNotAllowed` | PCR values not in allowlist |
| 46405 | `InvalidEnclaveAttestation` | Failed to parse attestation |
| 46406 | `BadFunctionName` | Function name mismatch |
| 46407 | `ReplayAttack` | Timestamp not newer than last accepted |
| 46408 | `NotGuardianRole` | Caller lacks Guardian role |
| 46409 | `SignerRegistrationRequired` | Must provide attestation for new signer |

---

## Data Types

### MeasurementRule

Defines an allowed enclave measurement:

```rust
#[odra::odra_type]
pub struct MeasurementRule {
    /// Platform identifier (e.g., "nitro" for AWS Nitro Enclaves)
    pub platform: String,

    /// Concatenated PCR values (PCR0.PCR1.PCR2)
    pub code: String,
}
```

### CachedSigner

Represents a verified and cached signer:

```rust
#[odra::odra_type]
pub struct CachedSigner {
    /// SEC1-encoded secp256k1 public key
    pub pubkey_sec1: Bytes,

    /// Platform that attested this key
    pub measurement_platform: String,

    /// PCR values for this key's enclave
    pub measurement_code: String,

    /// Block timestamp when registered
    pub registered_at: u64,

    /// Block timestamp of last successful report
    pub last_seen: u64,

    /// Whether this signer has been revoked
    pub revoked: bool,
}
```

### StyksBlockySupplierConfig

Contract configuration:

```rust
#[odra::odra_type]
pub struct StyksBlockySupplierConfig {
    /// Expected WASM hash of the Blocky guest program
    pub wasm_hash: String,

    /// Expected function name in attestation claims
    pub expected_function: String,

    /// List of allowed enclave measurements
    pub allowed_measurements: Vec<MeasurementRule>,

    /// Mapping from CoinGecko feed IDs to internal price feed IDs
    pub coingecko_feed_ids: Vec<(String, PriceFeedId)>,

    /// Address of the StyksPriceFeed contract
    pub price_feed_address: Address,

    /// Maximum age of price data in seconds
    pub timestamp_tolerance: u64,

    /// How long cached signers remain valid in seconds
    pub signer_ttl_secs: u64,
}
```

---

## Configuration Entrypoints

### set_config

Sets the contract configuration.

```rust
pub fn set_config(&mut self, config: StyksBlockySupplierConfig)
```

**Access**: `AdminRole` or `ConfigManagerRole`

**Parameters**:
- `config`: The new configuration

**Events**: None

**Errors**:
- `NotConfigManager`: Caller lacks required role

**Example**:
```rust
let config = StyksBlockySupplierConfig {
    wasm_hash: "baadaf...".to_string(),
    expected_function: "priceFunc".to_string(),
    allowed_measurements: vec![MeasurementRule {
        platform: "nitro".to_string(),
        code: "abc123...".to_string(),
    }],
    coingecko_feed_ids: vec![
        ("Gate_CSPR_USD".to_string(), "CSPRUSD".to_string())
    ],
    price_feed_address: feed_address,
    timestamp_tolerance: 1200,  // 20 minutes
    signer_ttl_secs: 86400,     // 24 hours
};
supplier.set_config(config);
```

---

## Signer Management Entrypoints

### register_signer

Registers a new signer from enclave attestation.

```rust
pub fn register_signer(
    &mut self,
    enclave_attestation: Bytes,
    public_key: Bytes,
) -> Bytes
```

**Access**: Public (anyone can register)

**Parameters**:
- `enclave_attestation`: JSON-encoded attestation claims
- `public_key`: SEC1-encoded public key bytes

**Returns**: `Bytes` - The signer ID (keccak256 hash of public key)

**Errors**:
- `ConfigNotSet`: Contract not configured
- `InvalidEnclaveAttestation`: Failed to parse attestation
- `MeasurementNotAllowed`: PCRs not in allowlist

**Notes**:
- Does not require any role (permissionless)
- If signer already exists and is not revoked, returns existing ID
- Revoked signers cannot be re-registered with the same key

**Example**:
```rust
let claims_json = blocky_output.enclave_attestation_claims_json();
let pubkey = blocky_output.public_key_bytes();
let signer_id = supplier.register_signer(
    Bytes::from(claims_json),
    Bytes::from(pubkey),
);
```

### revoke_signer

Revokes a signer, preventing future price reports.

```rust
pub fn revoke_signer(&mut self, signer_id: Bytes)
```

**Access**: `AdminRole` or `GuardianRole`

**Parameters**:
- `signer_id`: The signer ID to revoke

**Errors**:
- `NotGuardianRole`: Caller lacks required role
- `SignerNotFound`: Signer ID not in cache

**Notes**:
- Revocation is permanent (cannot be undone)
- The enclave must generate a new key to continue reporting

**Example**:
```rust
// Revoke a compromised signer
supplier.revoke_signer(compromised_signer_id);
```

---

## Price Reporting Entrypoints

### report_prices

Reports prices from Blocky attestation.

```rust
pub fn report_prices(
    &mut self,
    transitive_attestation: Bytes,
    signer_id: Option<Bytes>,
    enclave_attestation: Option<Bytes>,
    public_key: Option<Bytes>,
)
```

**Access**: Public (signature verification provides access control)

**Parameters**:
- `transitive_attestation`: ABI-encoded `bytes[]` containing signed price data
- `signer_id`: Optional cached signer ID for fast path
- `enclave_attestation`: Optional attestation claims for slow path
- `public_key`: Optional public key for slow path

**Usage Patterns**:

1. **Fast Path** (cached signer):
   ```rust
   supplier.report_prices(ta_bytes, Some(signer_id), None, None);
   ```

2. **Slow Path** (inline registration):
   ```rust
   supplier.report_prices(ta_bytes, None, Some(claims), Some(pubkey));
   ```

**Errors**:
- `ContractPaused`: Contract is paused
- `ConfigNotSet`: Contract not configured
- `SignerRegistrationRequired`: No signer_id and no attestation provided
- `SignerNotFound`: Signer ID not in cache
- `SignerRevoked`: Signer has been revoked
- `SignerExpired`: Signer TTL exceeded
- `MeasurementNotAllowed`: PCRs not in allowlist (slow path)
- `InvalidEnclaveAttestation`: Failed to parse attestation (slow path)
- `InvalidSignature`: Signature verification failed
- `HashMismatch`: WASM hash mismatch
- `BadFunctionName`: Function name mismatch
- `TimestampTooOld`: Price timestamp exceeds tolerance
- `ReplayAttack`: Timestamp not newer than last accepted
- `PriceFeedIdMismatch`: Unknown price feed ID

**Flow**:

```
1. Check not paused
2. Load config
3. Decode transitive attestation -> (data, signature)
4. Resolve public key:
   - Fast: Load from cache by signer_id
   - Slow: Parse attestation, verify PCRs, register
5. Verify ECDSA signature
6. Decode claims from data
7. Verify WASM hash matches config
8. Verify function name matches config
9. Extract price output
10. Verify timestamp within tolerance
11. Map CoinGecko feed ID to internal ID
12. Check replay protection (timestamp > last_accepted)
13. Forward price to StyksPriceFeed
14. Update last_accepted_timestamp
15. Update signer last_seen
```

**Example**:
```rust
// Fast path with cached signer
let ta_bytes = blocky_output.transitive_attestation_bytes();
let result = supplier.try_report_prices(
    Bytes::from(ta_bytes),
    Some(cached_signer_id.clone()),
    None,
    None,
);

match result {
    Ok(_) => println!("Price reported successfully"),
    Err(e) => {
        // Fall back to slow path
        let claims = blocky_output.enclave_attestation_claims_json();
        let pubkey = blocky_output.public_key_bytes();
        supplier.report_prices(
            Bytes::from(ta_bytes),
            None,
            Some(Bytes::from(claims)),
            Some(Bytes::from(pubkey)),
        );
    }
}
```

---

## Operational Control Entrypoints

### pause

Pauses the contract, preventing price reports.

```rust
pub fn pause(&mut self)
```

**Access**: `AdminRole` or `GuardianRole`

**Effects**:
- Sets `paused` flag to `true`
- All `report_prices` calls will revert

**Errors**:
- `NotGuardianRole`: Caller lacks required role

### unpause

Resumes contract operations.

```rust
pub fn unpause(&mut self)
```

**Access**: `AdminRole` or `GuardianRole`

**Effects**:
- Sets `paused` flag to `false`
- `report_prices` calls resume working

**Errors**:
- `NotGuardianRole`: Caller lacks required role

---

## Query Entrypoints

### get_config

Returns the current configuration.

```rust
pub fn get_config(&self) -> StyksBlockySupplierConfig
```

**Returns**: Current configuration

**Errors**:
- `ConfigNotSet`: Contract not configured

### get_config_or_none

Returns the current configuration or None.

```rust
pub fn get_config_or_none(&self) -> Option<StyksBlockySupplierConfig>
```

**Returns**: `Some(config)` if configured, `None` otherwise

### is_paused

Returns whether the contract is paused.

```rust
pub fn is_paused(&self) -> bool
```

**Returns**: `true` if paused, `false` otherwise

### get_signer

Returns information about a cached signer.

```rust
pub fn get_signer(&self, signer_id: Bytes) -> Option<CachedSigner>
```

**Parameters**:
- `signer_id`: The signer ID to look up

**Returns**: `Some(signer)` if found, `None` otherwise

### get_last_accepted_timestamp

Returns the last accepted timestamp for a price feed.

```rust
pub fn get_last_accepted_timestamp(&self, feed_id: PriceFeedId) -> Option<u64>
```

**Parameters**:
- `feed_id`: The price feed ID

**Returns**: `Some(timestamp)` if any price reported, `None` otherwise

---

## Gas Recommendations

| Operation | Recommended Gas |
|-----------|----------------|
| `set_config` | 3.5 - 4.0 CSPR |
| `register_signer` | 5.0 CSPR |
| `report_prices` (fast path) | 4.0 CSPR |
| `report_prices` (slow path) | 5.0 CSPR |
| `pause` / `unpause` | 1.0 CSPR |
| `revoke_signer` | 1.0 CSPR |

---

## Events

The contract does not emit custom events. Monitor state changes via:
- Query `get_signer` for registration status
- Query `get_last_accepted_timestamp` for price updates
- Query `is_paused` for operational status
