# Measurement-Anchored Model for Styks Oracle

## Overview

The Measurement-Anchored Model is a significant architectural upgrade to the `StyksBlockySupplier` contract that replaces the static "pinned public key" approach with a dynamic, attestation-based signer verification system.

### The Problem with Pinned Public Keys

In the original design, the `StyksBlockySupplier` contract stored a static public key used to verify Blocky attestation signatures:

```rust
// OLD: Static public key in config
pub struct StyksBlockySupplierConfig {
    pub wasm_hash: String,
    pub public_key: Bytes,  // <-- Static, requires governance to update
    // ...
}
```

This approach had a critical limitation: **every time the Blocky enclave restarted**, it generated a new signing key pair. This required a governance action to update the on-chain public key, creating operational friction and potential downtime.

### The Solution: Measurement-Anchored Trust

Instead of trusting a specific public key, we now trust a specific **enclave measurement**. The measurement is a cryptographic hash of the enclave's code and configuration (PCR values in AWS Nitro terminology).

```rust
// NEW: Trust measurements, not keys
pub struct StyksBlockySupplierConfig {
    pub wasm_hash: String,
    pub expected_function: String,
    pub allowed_measurements: Vec<MeasurementRule>,  // <-- Trust measurements
    pub signer_ttl_secs: u64,
    // ...
}
```

When a new Blocky instance starts:
1. It generates a fresh signing key
2. AWS Nitro creates an attestation document binding this key to the enclave measurement
3. The contract verifies the attestation and caches the signer if the measurement matches the allowlist
4. Subsequent price updates use the cached signer for fast verification

---

## Architecture Changes

### New Data Structures

#### MeasurementRule

Defines an allowed enclave measurement:

```rust
#[odra::odra_type]
pub struct MeasurementRule {
    /// Platform identifier (e.g., "nitro" for AWS Nitro Enclaves)
    pub platform: String,
    /// Measurement code: "pcr0hex.pcr1hex.pcr2hex" for Nitro
    pub code: String,
}
```

#### CachedSigner

Stores a verified enclave signing key:

```rust
#[odra::odra_type]
pub struct CachedSigner {
    /// SEC1-encoded public key (65 bytes for secp256k1 uncompressed)
    pub pubkey_sec1: Bytes,
    /// Measurement platform (e.g., "nitro")
    pub measurement_platform: String,
    /// Measurement code that was verified
    pub measurement_code: String,
    /// Timestamp when registered
    pub registered_at: u64,
    /// Timestamp of last successful price submission
    pub last_seen: u64,
    /// Whether this signer has been revoked
    pub revoked: bool,
}
```

#### Updated Config

```rust
#[odra::odra_type]
pub struct StyksBlockySupplierConfig {
    /// Expected WASM hash of the guest program
    pub wasm_hash: String,
    /// Expected function name (e.g., "priceFunc")
    pub expected_function: String,
    /// List of allowed enclave measurements
    pub allowed_measurements: Vec<MeasurementRule>,
    /// Mapping from CoinGecko IDs to PriceFeedIds
    pub coingecko_feed_ids: Vec<(String, PriceFeedId)>,
    /// Address of StyksPriceFeed contract
    pub price_feed_address: Address,
    /// Timestamp tolerance in seconds
    pub timestamp_tolerance: u64,
    /// TTL for cached signers (0 = no expiry)
    pub signer_ttl_secs: u64,
}
```

### New Storage

```rust
#[odra::module]
pub struct StyksBlockySupplier {
    access_control: SubModule<AccessControl>,
    config: Var<StyksBlockySupplierConfig>,
    /// Cached signers keyed by signer_id (hash of pubkey)
    cached_signers: Mapping<Bytes, CachedSigner>,
    /// Last accepted timestamp per feed (replay protection)
    last_accepted_timestamp: Mapping<PriceFeedId, u64>,
    /// Contract pause state
    paused: Var<bool>,
}
```

### New Role: Guardian

A new security role for emergency operations:

```rust
pub enum StyksBlockySupplierRole {
    Admin,          // DEFAULT_ADMIN_ROLE
    ConfigManager,  // [3u8; 32]
    Guardian,       // [4u8; 32] - NEW
}
```

The Guardian role can:
- Pause/unpause the contract
- Revoke compromised signers
- Register signers manually (in fallback mode)

---

## New Entrypoints

### Pause/Unpause

Emergency circuit breaker for the contract:

```rust
/// Pauses the contract (Guardian or Admin only)
pub fn pause(&mut self);

/// Unpauses the contract (Guardian or Admin only)
pub fn unpause(&mut self);

/// Returns whether the contract is paused
pub fn is_paused(&self) -> bool;
```

### Signer Management

```rust
/// Revokes a signer (Guardian or Admin only)
/// Revoked signers cannot submit prices
pub fn revoke_signer(&mut self, signer_id: Bytes);

/// Returns cached signer info if it exists
pub fn get_signer(&self, signer_id: Bytes) -> Option<CachedSigner>;
```

### Signer Registration (Hybrid Strategy)

Two registration methods are provided:

```rust
/// Permissionless on-chain verification (strict mode)
/// Currently returns OnChainAttestationVerificationDisabled
/// Will be enabled when COSE + X.509 verification is implemented
pub fn register_signer(&mut self, enclave_attestation: Bytes) -> Bytes;

/// Guardian/Admin only registration (fallback mode)
/// The CLI verifies attestation off-chain, then calls this
pub fn register_signer_manual(
    &mut self,
    pubkey_sec1: Bytes,
    measurement_platform: String,
    measurement_code: String,
) -> Bytes;
```

### Price Reporting

The new main entrypoint replacing `report_signed_prices`:

```rust
/// Reports prices from a transitive attestation
///
/// # Arguments
/// * `transitive_attestation` - Base64-decoded TA blob
/// * `signer_id` - Optional cached signer ID (fast path)
/// * `enclave_attestation` - Optional attestation (slow path)
///
/// At least one of signer_id or enclave_attestation required
pub fn report_prices(
    &mut self,
    transitive_attestation: Bytes,
    signer_id: Option<Bytes>,
    enclave_attestation: Option<Bytes>,
);
```

#### Fast Path vs Slow Path

**Fast Path** (recommended):
1. PriceProducer registers signer once on startup
2. Uses `signer_id` for subsequent price updates
3. Minimal gas cost, instant verification

**Slow Path** (fallback):
1. Each price update includes `enclave_attestation`
2. Contract verifies attestation and caches signer
3. Higher gas cost, but self-healing

---

## Security Features

### Replay Protection

Each price feed tracks the last accepted timestamp:

```rust
last_accepted_timestamp: Mapping<PriceFeedId, u64>
```

A price update is rejected if its timestamp is less than or equal to the last accepted timestamp for that feed.

### Signer TTL

Signers can expire after a configurable time:

```rust
signer_ttl_secs: u64  // 0 = no expiry, otherwise seconds
```

When a signer expires, the CLI automatically re-registers.

### Revocation

Guardians can revoke compromised signers:

```rust
supplier.revoke_signer(compromised_signer_id);
```

Revoked signers cannot submit prices until re-registered.

### Contract Pause

In emergencies, Guardians can pause all operations:

```rust
supplier.pause();
// ... investigate issue ...
supplier.unpause();
```

---

## Parser Modules

### transitive_attestation.rs

Decodes transitive attestation blobs:

```rust
pub struct DecodedTransitiveAttestation {
    pub data: Vec<u8>,        // Claims payload
    pub signature_rs: Vec<u8>, // 64-byte ECDSA signature (r, s)
}

pub fn decode_transitive_attestation(
    ta: &[u8]
) -> Result<DecodedTransitiveAttestation, TransitiveAttestationError>;
```

The TA blob is ABI-encoded as `bytes[]` with exactly 2 elements:
- Element 0: Data (claims payload)
- Element 1: Signature (65 bytes: r + s + v, we keep only r + s)

### nitro.rs

Parses AWS Nitro enclave attestations:

```rust
pub struct VerifiedEnclaveKey {
    pub measurement_platform: String,  // "nitro"
    pub measurement_code: String,      // "pcr0.pcr1.pcr2"
    pub pubkey_sec1: Vec<u8>,          // 65-byte SEC1 pubkey
}

pub struct NitroAttestationDocument {
    pub pcr0: Vec<u8>,      // Enclave image hash
    pub pcr1: Vec<u8>,      // Kernel + bootstrap hash
    pub pcr2: Vec<u8>,      // Application hash
    pub user_data: Vec<u8>, // Application public key
    pub certificate: Vec<u8>,
    pub cabundle: Vec<Vec<u8>>,
}

pub fn verify_nitro_enclave_attestation(
    doc: &[u8]
) -> Result<VerifiedEnclaveKey, EnclaveAttestationError>;

pub fn parse_nitro_attestation(
    cose_bytes: &[u8]
) -> Result<NitroAttestationDocument, EnclaveAttestationError>;

pub fn extract_measurement_from_attestation(
    enclave_attestation_b64: &str
) -> Result<(String, String, Vec<u8>), EnclaveAttestationError>;
```

The attestation format is JSON wrapped:
```json
{
  "platform": "nitro",
  "platform_attestations": ["<base64 COSE_Sign1>", ...]
}
```

Each platform attestation is a COSE_Sign1 document containing a CBOR attestation document.

### AWS Nitro Root Certificate

The AWS Nitro Enclaves Root CA is embedded for future strict-mode verification:

```rust
pub const AWS_NITRO_ROOT_CERT_DER: &[u8] =
    include_bytes!("../resources/AWS_NitroEnclaves_Root-G1.der");
```

SHA-256 fingerprint: `64:1A:03:21:A3:E2:44:EF:E4:56:46:31:95:D6:06:31:7E:D7:CD:CC:3C:17:56:E0:98:93:F3:C6:8F:79:BB:5B`

---

## CLI Updates

### set_config.rs

Updated to use the new config structure:

```rust
let supplier_config = StyksBlockySupplierConfig {
    wasm_hash,
    expected_function: String::from("priceFunc"),
    allowed_measurements: vec![MeasurementRule {
        platform: platform.clone(),
        code: code.clone(),
    }],
    coingecko_feed_ids: vec![...],
    price_feed_address: feed_addr,
    timestamp_tolerance: 20 * 60,   // 20 minutes
    signer_ttl_secs: 24 * 60 * 60,  // 24 hours
};
```

### update_price.rs

Implements signer caching:

```rust
pub struct Updater {
    // ...
    cached_signer_id: Option<Bytes>,  // NEW
}

impl Updater {
    /// Registers signer on startup and caches the ID
    fn ensure_signer_registered(&mut self) -> Result<(), String>;

    /// Uses fast path with cached signer, re-registers on failure
    pub fn report_price_via_blocky_supplier(&mut self);
}
```

Startup flow:
1. Call Blocky to get attestation
2. Extract measurement and public key
3. Call `register_signer_manual` (Guardian role required)
4. Cache the returned `signer_id`

Price update flow:
1. Call Blocky to get transitive attestation
2. Call `report_prices(ta, Some(signer_id), None)` (fast path)
3. On failure (expired, revoked), re-register and retry

---

## Error Codes

New error variants added:

| Code | Name | Description |
|------|------|-------------|
| 46102 | NotGuardianRole | Caller lacks Guardian role |
| 46206 | BadFunctionName | Function name doesn't match config |
| 46350-46354 | TA* | Transitive attestation decoding errors |
| 46400 | EnclaveAttestationFailed | Attestation verification failed |
| 46401 | MeasurementNotAllowed | Measurement not in allowlist |
| 46402 | SignerNotFound | Cached signer not found |
| 46403 | SignerRevoked | Signer has been revoked |
| 46404 | SignerExpired | Signer TTL exceeded |
| 46405 | OnChainAttestationVerificationDisabled | Strict mode not yet enabled |
| 46500 | ReplayDetected | Timestamp not newer than last |
| 46600 | ContractPaused | Contract is paused |
| 46700 | MissingSignerResolution | Neither signer_id nor attestation provided |

---

## Migration Guide

### From Pinned Public Key to Measurement-Anchored

1. **Update Contract Configuration**
   ```rust
   // Remove: public_key
   // Add: expected_function, allowed_measurements, signer_ttl_secs
   ```

2. **Grant Guardian Role**
   ```rust
   supplier.grant_role(&StyksBlockySupplierRole::Guardian.role_id(), &guardian_addr);
   ```

3. **Update PriceProducer**
   - Use `register_signer_manual` instead of relying on static key
   - Use `report_prices` instead of `report_signed_prices`
   - Implement signer caching for efficiency

4. **Configure Allowed Measurements**
   - Get measurement from Blocky attestation
   - Add to `allowed_measurements` in config

### Backwards Compatibility

The `report_signed_prices` entrypoint is deprecated and will return `OnChainAttestationVerificationDisabled`. Migrate to `report_prices` with signer caching.

---

## Future Work

### Strict Mode

Currently operating in "fallback mode" where the CLI performs full attestation verification off-chain. Future work will enable full on-chain verification:

1. COSE signature verification using the enclave certificate
2. X.509 certificate chain verification to AWS Nitro root
3. Permissionless `register_signer` entrypoint

### Multi-Platform Support

The measurement structure is designed to support multiple TEE platforms:
- AWS Nitro Enclaves (current)
- Intel SGX (future)
- AMD SEV-SNP (future)

Each platform would have its own verification logic in the parser crate.
