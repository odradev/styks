# Security Model

This document describes the security properties, trust assumptions, and threat mitigations of the Measurement Anchored Model.

---

## Table of Contents

1. [Trust Assumptions](#trust-assumptions)
2. [Security Properties](#security-properties)
3. [Threat Model](#threat-model)
4. [Attack Vectors and Mitigations](#attack-vectors-and-mitigations)
5. [Verification Approach](#verification-approach)
6. [Operational Security](#operational-security)
7. [Limitations](#limitations)

---

## Trust Assumptions

### What We Trust

1. **AWS Nitro Enclave Platform**
   - PCR measurements accurately reflect enclave contents
   - AWS attestation infrastructure is not compromised
   - Enclave isolation guarantees hold

2. **Cryptographic Primitives**
   - secp256k1 ECDSA signatures are unforgeable
   - Keccak256 is collision-resistant
   - ABI encoding is deterministic

3. **Blocky Service**
   - Runs the expected guest program (verified via WASM hash)
   - Correctly fetches prices from CoinGecko
   - Signs data with enclave-attested keys

4. **Contract Administrators**
   - Configure correct measurement allowlist
   - Respond appropriately to security incidents
   - Do not collude with attackers

### What We Don't Trust

1. **Any Specific Signing Key**
   - Keys can rotate, expire, or be compromised
   - Trust derives from measurement, not key identity

2. **Network Transport**
   - All data is verified on-chain
   - Man-in-the-middle cannot forge signatures

3. **Price Data Freshness**
   - Enforced via timestamp tolerance
   - Replay protection prevents stale reuse

---

## Security Properties

### Property 1: Measurement-Gated Trust

Only public keys proven to originate from enclaves with allowlisted measurements can report prices.

**Enforcement**:
```
parse_attestation(claims) -> measurement
measurement in allowed_measurements OR reject
```

### Property 2: Signature Authenticity

Every price report must have a valid ECDSA signature from a trusted signer.

**Enforcement**:
```
verify_signature(data, signature, public_key) OR reject
```

### Property 3: Data Integrity

Price data cannot be modified after signing without invalidating the signature.

**Enforcement**:
```
signature covers: hash_of_code, function, arguments, output, timestamp
any modification -> signature invalid -> rejected
```

### Property 4: Replay Protection

The same price data cannot be submitted twice.

**Enforcement**:
```
new_timestamp > last_accepted_timestamp[feed_id] OR reject
```

### Property 5: Freshness Guarantee

Price data cannot be older than the configured tolerance.

**Enforcement**:
```
|current_time - price_timestamp| <= timestamp_tolerance OR reject
```

### Property 6: Signer Lifecycle Control

Compromised signers can be immediately revoked.

**Enforcement**:
```
signer.revoked == true -> reject
signer.registered_at + ttl < current_time -> reject (expired)
```

---

## Threat Model

### Adversary Capabilities

We assume an adversary who can:

1. Observe all network traffic
2. Submit arbitrary transactions to the blockchain
3. Compromise individual signing keys
4. Run their own enclaves (but with different measurements)
5. Control price data sources temporarily

### Adversary Goals

1. **Price Manipulation**: Submit false prices to the oracle
2. **Denial of Service**: Prevent legitimate price updates
3. **Value Extraction**: Exploit price discrepancies for profit

---

## Attack Vectors and Mitigations

### Attack 1: Forged Signatures

**Vector**: Attacker creates signatures without access to private key.

**Mitigation**: ECDSA security - computationally infeasible without the private key.

**Residual Risk**: None (cryptographic assumption).

---

### Attack 2: Key Compromise

**Vector**: Attacker obtains a legitimate signing key.

**Mitigation**:
- **Detection**: Monitor for anomalous price submissions
- **Response**: Guardian calls `revoke_signer(signer_id)`
- **Recovery**: Enclave generates new key, auto-registers

**Residual Risk**: Limited window of exploitation before detection.

---

### Attack 3: Measurement Spoofing

**Vector**: Attacker creates attestation with false PCR values.

**Mitigation**:
- PCR values come from AWS Nitro attestation
- Cannot be forged without compromising AWS infrastructure

**Residual Risk**: Relies on AWS Nitro security model.

---

### Attack 4: Replay Attack

**Vector**: Attacker resubmits old (but valid) price data.

**Mitigation**:
```rust
if output.timestamp <= self.last_accepted_timestamp.get(&feed_id).unwrap_or(0) {
    revert(ReplayAttack);
}
```

**Residual Risk**: None (strict monotonic timestamp check).

---

### Attack 5: Stale Price Injection

**Vector**: Attacker submits legitimately signed but outdated prices.

**Mitigation**:
```rust
let age = current_time.saturating_sub(output.timestamp);
if age > config.timestamp_tolerance {
    revert(TimestampTooOld);
}
```

**Residual Risk**: Window of `timestamp_tolerance` seconds.

---

### Attack 6: Rogue Enclave

**Vector**: Attacker runs modified enclave with malicious code.

**Mitigation**:
- Different code = different PCR measurements
- Measurements not in allowlist = rejected

**Residual Risk**: None if allowlist is correctly configured.

---

### Attack 7: WASM Hash Collision

**Vector**: Attacker creates different WASM with same hash.

**Mitigation**:
- WASM hash uses cryptographic hash function
- Collision resistance makes this infeasible

**Residual Risk**: None (cryptographic assumption).

---

### Attack 8: Function Name Bypass

**Vector**: Attacker uses different function in same WASM.

**Mitigation**:
```rust
if claims.function() != config.expected_function {
    revert(BadFunctionName);
}
```

**Residual Risk**: None (explicit function name check).

---

### Attack 9: Feed ID Manipulation

**Vector**: Attacker maps prices to wrong feed IDs.

**Mitigation**:
- Feed ID mapping is in signed claims
- Cannot be modified without invalidating signature

**Residual Risk**: None.

---

### Attack 10: Denial of Service via Pause

**Vector**: Malicious Guardian pauses contract indefinitely.

**Mitigation**:
- Admin can always unpause
- Guardian role should be given carefully
- Multi-sig recommended for Guardian

**Residual Risk**: Governance/social layer.

---

## Verification Approach

### PCR Extraction (Current Implementation)

The contract parses attestation JSON to extract PCR measurements but does not perform full cryptographic verification of the AWS attestation chain.

```
Attestation JSON -> Parse -> Extract PCRs -> Compare to allowlist
```

**Rationale**:
1. Full AWS attestation verification requires:
   - AWS root certificate validation
   - COSE signature verification
   - Certificate chain validation
   - ~100KB+ of additional code

2. PCR extraction provides:
   - Measurement verification
   - Lower gas costs
   - Simpler implementation
   - Sufficient security for most use cases

**Trade-off**: An attacker who can forge attestation JSON format could potentially bypass verification. However:
- The JSON structure is complex and undocumented
- Signature verification still prevents price manipulation
- Enclave key must be registered before use

### Full Verification (Alternative)

For higher security requirements, full cryptographic verification could be implemented:

```
COSE_Sign1 -> Verify AWS Signature -> Parse Certificate -> Validate Chain -> Extract PCRs
```

This would require:
- AWS Nitro root certificate embedded in contract
- COSE signature verification library (no_std)
- X.509 certificate parsing
- Significantly higher gas costs

---

## Operational Security

### Measurement Allowlist Management

**Recommendation**: Treat allowlist updates as security-critical.

1. Verify new measurements against known-good enclave builds
2. Test new measurements on testnet before mainnet
3. Maintain audit trail of allowlist changes
4. Consider time-locks for measurement additions

### Guardian Key Management

**Recommendation**: Use multi-signature for Guardian role.

1. Distribute Guardian keys across multiple parties
2. Require M-of-N signatures for pause/revoke
3. Monitor Guardian actions via events
4. Regular key rotation

### Incident Response

**Playbook for suspected compromise**:

1. **Immediate**: Guardian calls `pause()`
2. **Investigate**: Identify compromised signer
3. **Revoke**: Call `revoke_signer(signer_id)`
4. **Verify**: Check allowlist is correct
5. **Resume**: Call `unpause()`
6. **Post-mortem**: Document and improve

### Monitoring

**Recommended alerts**:

1. Price deviation > threshold from external sources
2. Rapid succession of price updates
3. Signer registration from unexpected measurements
4. Pause/unpause events
5. Signer revocation events

---

## Limitations

### Known Limitations

1. **No Cross-Price Validation**
   - Contract accepts any price within tolerance
   - Does not compare against other oracles
   - Application layer should implement sanity checks

2. **Single Source Dependency**
   - Relies on CoinGecko as price source
   - CoinGecko manipulation could affect prices
   - Consider multi-source aggregation for high-value applications

3. **Centralized Blocky Service**
   - Single operator runs Blocky infrastructure
   - Service unavailability prevents updates
   - Consider redundant deployments

4. **No Slashing**
   - Malicious behavior results in revocation only
   - No economic penalty for attackers
   - Relies on off-chain accountability

5. **Timestamp Granularity**
   - Block timestamp used for time checks
   - Potential for minor manipulation by validators
   - Not suitable for sub-second precision

### Assumptions That Could Fail

| Assumption | Failure Mode | Impact |
|------------|--------------|--------|
| AWS Nitro secure | Measurement spoofing | Critical |
| CoinGecko accurate | Wrong prices | High |
| Admin honest | Malicious config | Critical |
| Network available | Update delays | Medium |
| secp256k1 secure | Signature forgery | Critical |

---

## Security Checklist

Before deployment:

- [ ] Measurement allowlist verified against known builds
- [ ] Timestamp tolerance appropriate for use case
- [ ] Signer TTL balanced (security vs convenience)
- [ ] Guardian role assigned to trusted parties
- [ ] Admin keys secured (multi-sig recommended)
- [ ] Monitoring and alerting configured
- [ ] Incident response playbook documented
- [ ] Testnet validation complete
- [ ] Security audit performed (recommended)
