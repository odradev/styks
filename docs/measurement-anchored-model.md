# Measurement Anchored Model

## Overview

The Measurement Anchored Model is a trust architecture for the Styks price oracle that replaces static public key verification with dynamic trust based on AWS Nitro Enclave measurements. This document explains the architecture, rationale, and implementation details.

## Table of Contents

1. [The Problem: Static Key Trust](#the-problem-static-key-trust)
2. [The Solution: Measurement Anchored Trust](#the-solution-measurement-anchored-trust)
3. [Architecture](#architecture)
4. [Data Flow](#data-flow)
5. [Components](#components)
6. [Configuration](#configuration)
7. [Operational Controls](#operational-controls)

---

## The Problem: Static Key Trust

### Original Design

The original `StyksBlockySupplier` contract stored a static public key in its configuration:

```rust
pub struct StyksBlockySupplierConfig {
    pub wasm_hash: String,
    pub public_key: String,  // Static, pinned public key
    pub coingecko_feed_ids: Vec<(String, PriceFeedId)>,
    pub price_feed_address: Address,
    pub timestamp_tolerance: u64,
}
```

Every price update required a signature that verified against this pinned key.

### The Key Rotation Problem

This design creates operational challenges:

1. **No Key Rotation**: If the Blocky service needs to rotate its signing key (security best practice, key compromise, hardware refresh), the on-chain contract must be reconfigured.

2. **Downtime Risk**: Key rotation requires a governance transaction, creating a window where price updates fail.

3. **Single Point of Failure**: One compromised key means all historical trust assumptions are broken.

4. **Coordination Overhead**: Every key change requires synchronization between Blocky operators and contract administrators.

---

## The Solution: Measurement Anchored Trust

### Core Insight

Instead of trusting a specific key, trust the **enclave measurement** (PCR values) that produced the key. Any key proven to originate from an enclave with allowlisted measurements is automatically trusted.

### How It Works

```
Traditional:
  Contract trusts: Public Key X
  Problem: Key X expires/rotates -> contract breaks

Measurement Anchored:
  Contract trusts: PCR measurements {PCR0, PCR1, PCR2}
  Any key from enclave matching those PCRs -> automatically trusted
```

### Benefits

| Aspect | Static Key | Measurement Anchored |
|--------|-----------|---------------------|
| Key Rotation | Requires governance tx | Automatic |
| Downtime | Yes, during rotation | None |
| Trust Scope | Single key | Any enclave instance |
| Flexibility | Low | High |

---

## Architecture

### System Components

```
+-------------------+     +------------------+     +-------------------+
|   Blocky Enclave  |     | StyksBlocky      |     | StyksPriceFeed    |
|   (AWS Nitro)     |---->| Supplier         |---->| Contract          |
|                   |     | Contract         |     |                   |
| - Fetches prices  |     | - Verifies sigs  |     | - Stores TWAP     |
| - Signs with key  |     | - Checks PCRs    |     | - Serves prices   |
| - Attests key     |     | - Caches signers |     |                   |
+-------------------+     +------------------+     +-------------------+
```

### Trust Chain

```
AWS Nitro Root of Trust
         |
         v
   Enclave Measurement (PCR0.PCR1.PCR2)
         |
         v
   Attestation Document (signed by AWS)
         |
         v
   Public Key (embedded in attestation)
         |
         v
   Price Signature (verified on-chain)
```

### Data Structures

#### MeasurementRule

Defines an allowlisted enclave configuration:

```rust
pub struct MeasurementRule {
    pub platform: String,  // "nitro" for AWS Nitro Enclaves
    pub code: String,      // "PCR0.PCR1.PCR2" concatenated
}
```

#### CachedSigner

Stores a verified signer for fast-path lookups:

```rust
pub struct CachedSigner {
    pub pubkey_sec1: Bytes,           // SEC1-encoded public key
    pub measurement_platform: String,  // Platform that attested this key
    pub measurement_code: String,      // PCR values for this key
    pub registered_at: u64,           // Block time of registration
    pub last_seen: u64,               // Last successful price report
    pub revoked: bool,                // Emergency revocation flag
}
```

#### Configuration

```rust
pub struct StyksBlockySupplierConfig {
    pub wasm_hash: String,                           // Expected WASM hash
    pub expected_function: String,                   // "priceFunc"
    pub allowed_measurements: Vec<MeasurementRule>,  // Allowlisted PCRs
    pub coingecko_feed_ids: Vec<(String, PriceFeedId)>,
    pub price_feed_address: Address,
    pub timestamp_tolerance: u64,                    // Max age of price data
    pub signer_ttl_secs: u64,                       // Signer cache duration
}
```

---

## Data Flow

### Signer Registration Flow

```
1. CLI starts price updater
         |
         v
2. Call Blocky to get fresh attestation
         |
         v
3. Extract enclave_attestation claims JSON + public key bytes
         |
         v
4. Call contract.register_signer(claims_json, pubkey_bytes)
         |
         v
5. Contract parses claims, extracts PCRs
         |
         v
6. Contract checks PCRs against allowed_measurements
         |
         v
7. If match: compute signer_id = keccak256(pubkey), cache signer
         |
         v
8. Return signer_id to CLI (cached for future use)
```

### Price Reporting Flow (Fast Path)

When a signer is already registered:

```
1. CLI calls Blocky for fresh price + signature
         |
         v
2. CLI calls contract.report_prices(
       transitive_attestation,
       signer_id: Some(cached_id),
       enclave_attestation: None,
       pubkey: None
   )
         |
         v
3. Contract looks up signer by ID
         |
         v
4. Contract verifies: not revoked, not expired, signature valid
         |
         v
5. Contract decodes price claims, verifies WASM hash + function
         |
         v
6. Contract checks timestamp tolerance + replay protection
         |
         v
7. Contract forwards price to StyksPriceFeed
```

### Price Reporting Flow (Slow Path)

When signer is expired, revoked, or not yet registered:

```
1. Fast path fails (signer not found or expired)
         |
         v
2. CLI calls contract.report_prices(
       transitive_attestation,
       signer_id: None,
       enclave_attestation: Some(claims_json),
       pubkey: Some(pubkey_bytes)
   )
         |
         v
3. Contract registers signer inline (same as register_signer)
         |
         v
4. Contract continues with signature verification + price forwarding
         |
         v
5. CLI caches new signer_id for future fast-path calls
```

---

## Components

### Parser Layer (`styks-blocky-parser`)

#### Transitive Attestation Decoder

File: `src/transitive_attestation.rs`

Decodes ABI-encoded `bytes[]` containing signed price data:

```rust
pub fn decode_transitive_attestation(ta_bytes: &[u8])
    -> Result<(Vec<u8>, Vec<u8>), TAError>
```

Returns:
- `data`: The signed claims (function call result)
- `signature`: 64-byte ECDSA signature (r || s)

#### Enclave Attestation Parser

File: `src/enclave_attestation.rs`

Parses enclave attestation claims to extract measurements:

```rust
pub fn parse_enclave_attestation_with_pubkey(
    claims_json: &[u8],
    public_key_bytes: Vec<u8>,
) -> Result<EnclaveAttestation, EnclaveAttestationError>
```

Returns:
```rust
pub struct EnclaveAttestation {
    pub platform: String,         // "nitro"
    pub measurement_code: String, // "pcr0.pcr1.pcr2"
    pub public_key: Vec<u8>,      // SEC1 bytes
}
```

#### BlockyOutput Helpers

File: `src/blocky_output.rs`

Helper methods for extracting raw bytes:

```rust
impl BlockyOutput {
    pub fn transitive_attestation_bytes(&self) -> Vec<u8>
    pub fn enclave_attestation_bytes(&self) -> Vec<u8>
    pub fn enclave_attestation_claims_json(&self) -> Vec<u8>
    pub fn public_key_bytes(&self) -> Vec<u8>
}
```

### Contract Layer (`styks-contracts`)

File: `src/styks_blocky_supplier.rs`

See [API Reference](./api-reference.md) for detailed entrypoint documentation.

### CLI Layer (`styks-cli`)

#### SetConfig Scenario

File: `src/scenarios/set_config.rs`

Configures contracts with measurement rules extracted from Blocky output:

```rust
let supplier_config = StyksBlockySupplierConfig {
    wasm_hash,
    expected_function: String::from("priceFunc"),
    allowed_measurements: vec![MeasurementRule {
        platform: measurement.platform.clone(),
        code: measurement.code.clone(),
    }],
    coingecko_feed_ids: vec![...],
    price_feed_address: feed_addr,
    timestamp_tolerance: 20 * 60,  // 20 minutes
    signer_ttl_secs: 24 * 60 * 60, // 24 hours
};
```

#### UpdatePrice Scenario

File: `src/scenarios/update_price.rs`

Manages signer lifecycle and price updates:

```rust
struct Updater {
    cached_signer_id: Option<Bytes>,  // Cached for fast-path
    // ...
}
```

- Registers signer on startup
- Uses fast path when signer is cached
- Falls back to slow path on errors
- Re-registers on expiry/revocation

---

## Configuration

### Measurement Rules

Measurements are extracted from the Blocky output's attestation claims:

```json
{
  "enclave_measurement": {
    "platform": "nitro",
    "code": "abc123...def456...789ghi..."
  }
}
```

The `code` field contains concatenated PCR values (PCR0.PCR1.PCR2).

### Signer TTL

`signer_ttl_secs` controls how long a cached signer remains valid:

- **24 hours** (86400): Recommended for production
- Shorter values increase security but require more frequent attestation
- Expired signers automatically fall back to slow path

### Timestamp Tolerance

`timestamp_tolerance` controls how old price data can be:

- **20 minutes** (1200): Recommended default
- Prevents stale price injection
- Must be balanced with network latency and heartbeat intervals

---

## Operational Controls

### Pause/Unpause

Guardians or Admins can pause the contract in emergencies:

```rust
contract.pause();    // Stop all price updates
contract.unpause();  // Resume operations
```

When paused:
- `report_prices` reverts with `ContractPaused`
- `register_signer` still works (prepare for resume)
- Query functions remain available

### Signer Revocation

Revoke a compromised signer immediately:

```rust
contract.revoke_signer(signer_id);
```

Effects:
- Signer marked as `revoked: true`
- All future `report_prices` with this signer fail
- Signer cannot be un-revoked (must register new key)

### Guardian Role

The Guardian role (`[4u8; 32]`) has limited operational powers:

- Can pause/unpause
- Can revoke signers
- Cannot change configuration
- Cannot grant/revoke roles

This separation allows rapid incident response without full admin access.

---

## Migration from Static Key Model

### Steps

1. **Deploy new contract** with measurement-anchored config
2. **Configure measurements** from current Blocky enclave
3. **Update CLI** to use new registration flow
4. **Test on testnet** with full signer lifecycle
5. **Deploy to mainnet** with initial signer registration

### Breaking Changes

- `public_key` field removed from config
- `report_signed_prices` replaced with `report_prices`
- New `register_signer` entrypoint
- New error codes (46400-46408)

### Backwards Compatibility

None. This is a clean break. The old `report_signed_prices` entrypoint no longer exists.
