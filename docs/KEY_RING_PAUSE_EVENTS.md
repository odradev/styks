# Key Ring, Pause Circuit Breaker, and Events

This document describes the security enhancements added to `StyksBlockySupplier` for production-grade key management, emergency response, and on-chain monitoring.

## Table of Contents

- [Overview](#overview)
- [Problem Statement](#problem-statement)
- [Solution](#solution)
- [Features](#features)
  - [Key Ring Management](#key-ring-management)
  - [Guardian Role](#guardian-role)
  - [Pause Circuit Breaker](#pause-circuit-breaker)
  - [On-Chain Events](#on-chain-events)
  - [Function Name Enforcement](#function-name-enforcement)
  - [Monotonic Timestamp Anti-Replay](#monotonic-timestamp-anti-replay)
- [API Reference](#api-reference)
- [Error Codes](#error-codes)
- [Deployment Guide](#deployment-guide)
- [Operational Procedures](#operational-procedures)
- [Risks and Mitigations](#risks-and-mitigations)
- [Benefits](#benefits)
- [Backward Compatibility](#backward-compatibility)
- [File Changes Summary](#file-changes-summary)

---

## Overview

The Blocky Attestation Service generates a fresh enclave keypair on each startup or redeploy. The original single-key design meant any key change would break price reporting until the contract config was updated, creating downtime windows.

This enhancement adds:

- **Key Ring**: Multi-key allowlist with time-bounded validity and revocation
- **Guardian Role**: Separate emergency-response role for operational safety
- **Pause Circuit Breaker**: Immediate halt capability for emergencies
- **On-Chain Events**: Transparent audit trail for all security operations
- **Function Name Enforcement**: Validates the Blocky guest function name
- **Monotonic Timestamps**: Anti-replay protection per price feed

---

## Problem Statement

### Original Design Limitations

```
StyksBlockySupplerConfig {
    public_key: Bytes,  // Single pinned key - no rotation support
    ...
}
```

**Issues:**

1. **No Zero-Downtime Rotation**: Blocky AS restarts generate new keys. Old key in config means all price reports rejected until `set_config()` is called.

2. **No Emergency Revocation**: If a key is compromised, the only option is full config update, which is slow and requires the ConfigManager.

3. **No Circuit Breaker**: No way to immediately halt operations if something goes wrong.

4. **No Audit Trail**: No on-chain events for security-relevant operations.

5. **Single Admin Bottleneck**: ConfigManager handles both routine config and emergency response.

6. **Replay Vulnerability**: Same signed data could potentially be replayed.

---

## Solution

### Architecture

```
+------------------+     +-------------------+     +------------------+
|   Blocky AS      | --> | StyksBlocky       | --> | StyksPriceFeed   |
|   (signs data)   |     | Supplier          |     | (stores prices)  |
+------------------+     +-------------------+     +------------------+
                                |
                    +-----------+-----------+
                    |                       |
            +-------v-------+       +-------v-------+
            | Key Ring      |       | Pause State   |
            | [SignerKey1]  |       | is_paused     |
            | [SignerKey2]  |       +---------------+
            | ...           |
            +---------------+

Roles:
  - Admin: Manages all roles
  - ConfigManager: Routine config, key ring management
  - Guardian: Emergency pause/unpause, key revocation
```

### Key Design Decisions

1. **New Storage Fields Only**: Does NOT modify `StyksBlockySupplerConfig` serialization. Adds separate storage variables for upgrade safety.

2. **Backward Compatible**: Empty key ring falls back to `config.public_key`.

3. **Role Separation**: Guardian role handles emergencies; ConfigManager handles routine operations.

4. **Time-Bounded Keys**: Keys can have `not_before` and `not_after` timestamps for scheduled rotation.

---

## Features

### Key Ring Management

The key ring is a vector of `SignerKeyRecord` entries stored separately from the main config:

```rust
pub struct SignerKeyRecord {
    pub public_key: Bytes,  // SEC1 secp256k1 public key
    pub not_before: u64,    // Unix timestamp (0 = immediately active)
    pub not_after: u64,     // Unix timestamp (0 = no expiry)
    pub revoked: bool,      // Immediate invalidation flag
}
```

**Key Validity Logic:**

```rust
fn is_active(&self, now: u64) -> bool {
    if self.revoked { return false; }
    if self.not_before != 0 && now < self.not_before { return false; }
    if self.not_after != 0 && now > self.not_after { return false; }
    true
}
```

**Signature Verification Flow:**

1. If key ring is empty, use `config.public_key` (backward compatibility)
2. If key ring has entries, try each active key until one verifies
3. If no key verifies, revert with `BadSignature`

### Guardian Role

New role specifically for emergency operations:

| Role | ID | Capabilities |
|------|-----|--------------|
| Admin | `[0u8; 32]` | Manage all roles |
| ConfigManager | `[3u8; 32]` | Config, add/retire keys |
| Guardian | `[4u8; 32]` | Pause/unpause, revoke keys |

**Why Separate Roles?**

- ConfigManager may be a multisig or DAO with slow execution
- Guardian can be a hot wallet for immediate emergency response
- Principle of least privilege: Guardian cannot change config

### Pause Circuit Breaker

Immediate halt for all price reporting:

```rust
pub fn pause(&mut self)    // Guardian or Admin only
pub fn unpause(&mut self)  // Guardian or Admin only
pub fn is_paused(&self) -> bool
```

When paused, `report_signed_prices()` reverts immediately with `ContractPaused`.

**Use Cases:**

- Suspected key compromise
- Blocky AS malfunction
- Upstream data source issues
- Protocol upgrade coordination

### On-Chain Events

All security operations emit events for monitoring and audit:

```rust
// Key ring events
SignerKeyAdded { by: Address, public_key: Bytes, not_before: u64, not_after: u64 }
SignerKeyRetired { by: Address, public_key: Bytes, not_after: u64 }
SignerKeyRevoked { by: Address, public_key: Bytes }

// Pause events
Paused { account: Address }
Unpaused { account: Address }
```

**Monitoring Integration:**

- Index events for alerting dashboards
- Detect unauthorized key changes
- Audit trail for compliance

### Function Name Enforcement

Validates the Blocky guest function name matches expectations:

```rust
pub fn set_expected_function(&mut self, name: String)  // ConfigManager
pub fn get_expected_function(&self) -> String
```

If `expected_function` is set (non-empty), `report_signed_prices` verifies:
```rust
if !expected_fn.is_empty() && claims.function() != expected_fn {
    self.env().revert(StyksBlockySupplerError::BadFunctionName);
}
```

Default: `"priceFunc"` (set by CLI during deployment)

### Monotonic Timestamp Anti-Replay

Prevents replay of old signed data:

```rust
// Storage
last_seen_timestamp: Mapping<PriceFeedId, u64>

// In report_signed_prices:
let last = self.last_seen_timestamp.get(&price_feed_id).unwrap_or_default();
if output.timestamp <= last {
    self.env().revert(StyksBlockySupplerError::TimestampNotMonotonic);
}
// After successful submission:
self.last_seen_timestamp.set(&price_feed_id, output.timestamp);
```

---

## API Reference

### Key Ring Management

| Function | Access | Description |
|----------|--------|-------------|
| `get_signer_keys() -> Vec<SignerKeyRecord>` | Public | Returns all key records |
| `add_signer_key(public_key, not_before, not_after)` | ConfigManager | Adds a key to the ring |
| `retire_signer_key(public_key, not_after)` | ConfigManager | Sets expiry on existing key |
| `revoke_signer_key(public_key)` | Guardian or ConfigManager | Immediately invalidates key |

### Pause Control

| Function | Access | Description |
|----------|--------|-------------|
| `pause()` | Guardian or Admin | Halts all price reporting |
| `unpause()` | Guardian or Admin | Resumes price reporting |
| `is_paused() -> bool` | Public | Returns pause state |

### Function Enforcement

| Function | Access | Description |
|----------|--------|-------------|
| `set_expected_function(name)` | ConfigManager | Sets required function name |
| `get_expected_function() -> String` | Public | Returns current setting |

---

## Error Codes

### New Errors

| Code | Name | Description |
|------|------|-------------|
| 46102 | `NotGuardianRole` | Caller lacks Guardian (or required) role |
| 46400 | `DuplicateSignerKey` | Key already exists in ring |
| 46401 | `SignerKeyNotFound` | Key not found in ring |
| 46402 | `BadFunctionName` | Claims function name mismatch |
| 46403 | `TimestampNotMonotonic` | Timestamp not newer than last seen |
| 46404 | `ContractPaused` | Contract is paused |

### Existing Errors (unchanged)

| Code | Name |
|------|------|
| 46000 | `ConfigNotSet` |
| 46001 | `PriceFeedIdNotFound` |
| 46100 | `NotAdminRole` |
| 46101 | `NotConfigManagerRole` |
| 46200 | `InvalidPublicKey` |
| 46201 | `InvalidSignature` |
| 46202 | `HashingError` |
| 46203 | `BadSignature` |
| 46204 | `BadWasmHash` |
| 46205 | `TimestampOutOfRange` |

---

## Deployment Guide

### Fresh Deployment

1. **Build contracts:**
   ```bash
   cargo odra build
   ```

2. **Deploy contracts** (standard Odra deployment)

3. **Run SetConfig scenario:**
   ```bash
   cargo run --bin styks-cli -- run SetConfig
   ```
   This automatically:
   - Sets contract configuration
   - Bootstraps key ring with current Blocky public key
   - Sets expected function to `"priceFunc"`

4. **Run SetPermissions scenario:**
   ```bash
   cargo run --bin styks-cli -- run SetPermissions \
     --guardian-address "account-hash-<GUARDIAN_ACCOUNT_HASH>"
   ```
   This grants:
   - ConfigManager role to deployer
   - PriceSupplier role to server account
   - Guardian role to specified address

### Upgrade Existing Deployment

Since new storage fields are added without modifying existing storage layout:

1. **Build new WASM:**
   ```bash
   cargo odra build
   ```

2. **Upgrade contract** using Casper upgrade mechanism

3. **Bootstrap key ring** (if not already done):
   ```rust
   // Call add_signer_key with current public key
   supplier.add_signer_key(current_public_key, 0, 0);
   ```

4. **Grant Guardian role:**
   ```rust
   supplier.grant_role(GUARDIAN_ROLE_ID, guardian_address);
   ```

---

## Operational Procedures

### Zero-Downtime Key Rotation

1. **Obtain new Blocky enclave public key** (from Blocky output)

2. **Add new key to ring:**
   ```rust
   supplier.add_signer_key(new_key, 0, 0);  // Immediately active
   ```

3. **Verify both keys work** - submit test price reports

4. **Switch Blocky AS to new key** (restart/redeploy)

5. **Confirm new key works** - monitor price submissions

6. **Retire old key:**
   ```rust
   let expiry = now + 3600;  // 1 hour grace period
   supplier.retire_signer_key(old_key, expiry);
   ```

### Scheduled Key Rotation

For planned rotations with known timing:

```rust
// Add new key that becomes active at specific time
supplier.add_signer_key(new_key, activation_time, 0);

// Set old key to expire at same time
supplier.retire_signer_key(old_key, activation_time);
```

### Emergency Key Revocation

If a key is suspected compromised:

1. **Immediately revoke the key:**
   ```rust
   supplier.revoke_signer_key(compromised_key);  // Guardian can do this
   ```

2. **Consider pausing if situation is unclear:**
   ```rust
   supplier.pause();
   ```

3. **Add replacement key if needed:**
   ```rust
   supplier.add_signer_key(new_key, 0, 0);
   ```

4. **Unpause when ready:**
   ```rust
   supplier.unpause();
   ```

### Emergency Pause

For any emergency requiring immediate halt:

1. **Pause the contract:**
   ```rust
   supplier.pause();  // Guardian hot wallet
   ```

2. **Investigate and remediate**

3. **Unpause when safe:**
   ```rust
   supplier.unpause();
   ```

---

## Risks and Mitigations

### Risk: Guardian Key Compromise

**Impact:** Attacker can pause contract or revoke keys (DoS), but cannot:
- Add new keys (requires ConfigManager)
- Change config
- Steal funds (none held)

**Mitigation:**
- Guardian should be a secure hot wallet
- Admin can revoke Guardian role
- Monitor Guardian events for unauthorized actions

### Risk: All Keys Revoked/Expired

**Impact:** No valid keys means no price reports accepted

**Mitigation:**
- Always add new key before removing/expiring old key
- Key ring falls back to `config.public_key` if empty
- Monitor key expiry times

### Risk: Key Ring Storage Growth

**Impact:** Unbounded key ring could increase gas costs

**Mitigation:**
- Periodically clean up revoked/expired keys
- Consider adding max key ring size limit in future

### Risk: Replay After Key Re-addition

**Impact:** If a key is revoked then re-added, old signatures might work

**Mitigation:**
- Monotonic timestamp check prevents replays
- Same timestamp cannot be submitted twice per feed

---

## Benefits

1. **Zero-Downtime Rotation**: Add new key, transition, remove old key - no service interruption

2. **Emergency Response**: Guardian can immediately pause or revoke without waiting for multisig

3. **Audit Trail**: All security operations emit events for monitoring and compliance

4. **Scheduled Rotation**: Time-bounded keys enable planned transitions

5. **Defense in Depth**: Multiple layers - key validation, function name check, timestamp monotonicity, pause gate

6. **Backward Compatible**: Existing deployments continue working; features are opt-in

7. **Role Separation**: Routine operations (ConfigManager) separated from emergency response (Guardian)

---

## Backward Compatibility

### Storage Compatibility

- **New storage variables** added alongside existing `config`:
  - `signer_keys: Var<Vec<SignerKeyRecord>>`
  - `is_paused: Var<bool>`
  - `last_seen_timestamp: Mapping<PriceFeedId, u64>`
  - `expected_function: Var<String>`

- **Existing storage unchanged**: `config: Var<StyksBlockySupplerConfig>` layout preserved

### Behavioral Compatibility

- **Empty key ring**: Falls back to `config.public_key`
- **Empty expected_function**: Function name not enforced
- **is_paused default**: `false` (not paused)
- **last_seen_timestamp default**: `0` (any timestamp accepted initially)

### API Compatibility

- **Existing entrypoints**: Signatures unchanged
- **`report_signed_prices`**: Same parameters, enhanced validation

---

## File Changes Summary

```
styks-contracts/src/styks_blocky_supplier.rs
  - Added SignerKeyRecord struct with is_active() method
  - Added 5 events: SignerKeyAdded, SignerKeyRetired, SignerKeyRevoked, Paused, Unpaused
  - Added Guardian role to StyksBlockySupplerRole enum
  - Added 6 new error variants (46102, 46400-46404)
  - Added 4 new storage fields
  - Added key ring entrypoints: get_signer_keys, add_signer_key, retire_signer_key, revoke_signer_key
  - Added pause entrypoints: pause, unpause, is_paused
  - Added function entrypoints: set_expected_function, get_expected_function
  - Modified report_signed_prices with pause gate, key ring verification, function check, monotonic timestamp
  - Added helper methods: assert_guardian_or_admin, require_not_paused, assert_valid_signature_any

styks-blocky-parser/src/verify.rs
  - Added validate_public_key() function for SEC1 key validation

styks-blocky-parser/src/blocky_claims.rs
  - Fixed test assertions to match current test data (price 516, timestamp 1765796826)

styks-cli/src/scenarios/set_permissions.rs
  - Added --guardian-address CLI argument
  - Grants Guardian role to specified address

styks-cli/src/scenarios/set_config.rs
  - Auto-bootstraps key ring with current public key after config
  - Sets expected function to "priceFunc" for new deployments
```

---

## Appendix: SignerKeyRecord Lifecycle

```
                    add_signer_key()
                          |
                          v
+------------------+     +------------------+
|   NOT EXISTS     | --> |     ACTIVE       |
+------------------+     | (or PENDING if   |
                         | not_before > now)|
                         +------------------+
                               |
              +----------------+----------------+
              |                                 |
              v                                 v
     retire_signer_key()              revoke_signer_key()
              |                                 |
              v                                 v
+------------------+                 +------------------+
|    RETIRING      |                 |     REVOKED      |
| (active until    |                 | (immediately     |
|  not_after)      |                 |  inactive)       |
+------------------+                 +------------------+
              |
              v (time passes)
+------------------+
|     EXPIRED      |
| (not_after < now)|
+------------------+
```
