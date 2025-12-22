# Changelog

All notable changes to the Styks Oracle project are documented here.

## [Unreleased] - 2025-12-22

### Added

#### Measurement-Anchored Model

A major architectural upgrade replacing the static "pinned public key" model with dynamic attestation-based signer verification.

**New Contract Features:**
- `MeasurementRule` struct for defining allowed enclave measurements
- `CachedSigner` struct for storing verified enclave signing keys
- `Guardian` role for emergency operations (pause, revoke, manual registration)
- `pause()` / `unpause()` entrypoints for emergency circuit breaker
- `revoke_signer()` entrypoint to revoke compromised signers
- `register_signer_manual()` entrypoint for CLI-verified signer registration
- `report_prices()` entrypoint with fast-path (cached signer) and slow-path (attestation) support
- Replay protection via `last_accepted_timestamp` per price feed
- Signer TTL support via `signer_ttl_secs` config option

**New Parser Modules:**
- `styks-blocky-parser/src/transitive_attestation.rs` - Decodes transitive attestation blobs
- `styks-blocky-parser/src/nitro.rs` - Parses AWS Nitro enclave attestations (COSE_Sign1 + CBOR)
- Embedded AWS Nitro Enclaves Root CA certificate

**New Config Fields:**
- `expected_function` - Expected function name in guest program
- `allowed_measurements` - List of allowed enclave measurements (replaces `public_key`)
- `signer_ttl_secs` - Time-to-live for cached signers

**CLI Updates:**
- `set_config.rs` - Uses new config structure with measurements
- `update_price.rs` - Implements signer caching with automatic re-registration

**New Dependencies:**
- `coset` - COSE parsing
- `ciborium` - CBOR parsing
- `p384` - P-384 ECDSA (for attestation certificates)
- `sha2` - SHA-256/384 hashing

### Changed

- `StyksBlockySupplierConfig` no longer contains `public_key` field
- `report_signed_prices()` is deprecated in favor of `report_prices()`

### Fixed

- Typos in contract names: `StyksBlockySuppler*` renamed to `StyksBlockySupplier*`

### Documentation

- Added `docs/measurement-anchored-model.md` - Complete architecture documentation
- Added `docs/nitro-attestation-format.md` - AWS Nitro attestation format details
- Updated `README.md` with new StyksBlockySupplier documentation

---

## [0.1.0] - 2025-08-17

### Added

- Initial release of Styks Oracle
- `StyksPriceFeed` contract for storing TWAP prices
- `StyksBlockySupplier` contract for Blocky integration
- Heartbeat scheduling mechanism
- TWAP calculation with missed heartbeat tolerance
- CLI tool for deployment and price updates
- Blocky guest program for CoinGecko price fetching
