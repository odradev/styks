# AWS Nitro Enclave Attestation Format

This document describes the attestation format used by AWS Nitro Enclaves and how Styks parses it for signer verification.

## Overview

AWS Nitro Enclaves use COSE (CBOR Object Signing and Encryption) to create signed attestation documents. These documents prove that code is running inside a genuine AWS Nitro enclave and bind application data (like public keys) to the enclave's identity.

## Attestation Structure

### Outer JSON Wrapper

The attestation begins as a JSON document:

```json
{
  "platform": "nitro",
  "platform_attestations": [
    "<base64-encoded COSE_Sign1 document>",
    ...
  ]
}
```

- `platform`: Always "nitro" for AWS Nitro Enclaves
- `platform_attestations`: Array of base64-encoded COSE_Sign1 documents

### COSE_Sign1 Structure

After base64 decoding, each platform attestation is a COSE_Sign1 document (RFC 8152):

```
COSE_Sign1 = [
    protected_header,   // CBOR map with algorithm info
    unprotected_header, // Usually empty
    payload,            // CBOR attestation document
    signature           // ECDSA-P384 signature
]
```

The document uses the **untagged** COSE_Sign1 format (starts with `0x84` array marker, not the tagged `0xD2` format).

### Attestation Document Payload

The payload is a CBOR map containing:

```cbor
{
  "module_id": "<enclave module ID>",
  "timestamp": <unix timestamp>,
  "digest": "SHA384",
  "pcrs": {
    0: <48 bytes>,  // Enclave image hash
    1: <48 bytes>,  // Linux kernel + bootstrap hash
    2: <48 bytes>,  // Application hash
    3: <48 bytes>,  // IAM role (if any)
    4: <48 bytes>,  // Instance ID (if any)
    ...             // Up to PCR 15
  },
  "certificate": <DER-encoded X.509 certificate>,
  "cabundle": [
    <DER-encoded intermediate cert>,
    ...
  ],
  "public_key": null or <bytes>,
  "user_data": <application-specific data>,
  "nonce": null or <bytes>
}
```

## PCR (Platform Configuration Register) Values

PCRs are SHA-384 hashes that identify the enclave configuration:

| PCR | Description |
|-----|-------------|
| 0 | Enclave Image File (EIF) hash |
| 1 | Linux kernel and bootstrap hash |
| 2 | Application (user code) hash |
| 3 | IAM role assigned to parent instance |
| 4 | Instance ID of parent instance |
| 5-15 | Reserved for future use |

### Measurement Code Format

Styks combines PCR 0, 1, and 2 into a measurement code:

```
pcr0hex.pcr1hex.pcr2hex
```

Example:
```
000102030405...abcdef.112233445566...fedcba.aabbccdd...11223344
```

Each PCR is 48 bytes (96 hex characters), so the total measurement code is 290 characters (96 + 1 + 96 + 1 + 96).

## user_data Field

The `user_data` field contains application-specific data embedded by the enclave. In Blocky's case, this contains the application's public key:

```json
{
  "curve_type": "p256k1",
  "data": "<base64-encoded SEC1 public key>"
}
```

The SEC1 public key is 65 bytes in uncompressed format:
- Byte 0: `0x04` (uncompressed point marker)
- Bytes 1-32: X coordinate
- Bytes 33-64: Y coordinate

## Certificate Chain

### Enclave Certificate

The `certificate` field contains a short-lived X.509 certificate:
- Issued by the CA in `cabundle`
- Contains the enclave's P-384 ECDSA public key
- Valid for a limited time window

### CA Bundle

The `cabundle` array contains intermediate certificates leading to the AWS Nitro Enclaves Root CA:

```
[enclave_cert] -> cabundle[0] -> cabundle[1] -> ... -> Root CA
```

### Root CA

AWS Nitro Enclaves Root CA (G1):
- Download: https://aws-nitro-enclaves.amazonaws.com/AWS_NitroEnclaves_Root-G1.zip
- SHA-256: `64:1A:03:21:A3:E2:44:EF:E4:56:46:31:95:D6:06:31:7E:D7:CD:CC:3C:17:56:E0:98:93:F3:C6:8F:79:BB:5B`

## Parsing in Styks

### Step 1: Parse JSON Wrapper

```rust
let wrapper: EnclaveAttestationJson = serde_json_wasm::from_str(json_str)?;
if wrapper.platform != "nitro" {
    return Err(EnclaveAttestationError::InvalidPlatform);
}
```

### Step 2: Decode Base64

```rust
let cose_bytes = BASE64_STANDARD.decode(wrapper.platform_attestations[0])?;
```

### Step 3: Parse COSE_Sign1

```rust
// Note: untagged format, not from_tagged_slice
let cose_sign1 = CoseSign1::from_slice(&cose_bytes)?;
let payload = cose_sign1.payload.ok_or(PayloadMissing)?;
```

### Step 4: Parse CBOR Attestation Document

```rust
let attestation_doc: CborValue = ciborium::from_reader(&payload[..])?;
let doc_map = match attestation_doc {
    CborValue::Map(m) => m,
    _ => return Err(AttestationDocumentNotMap),
};
```

### Step 5: Extract PCRs

```rust
let pcrs_map = get_field("pcrs")?.as_map()?;
let pcr0 = get_pcr(0)?;  // 48 bytes
let pcr1 = get_pcr(1)?;  // 48 bytes
let pcr2 = get_pcr(2)?;  // 48 bytes

let measurement_code = format!(
    "{}.{}.{}",
    hex::encode(&pcr0),
    hex::encode(&pcr1),
    hex::encode(&pcr2)
);
```

### Step 6: Extract Public Key from user_data

```rust
let user_data = get_field("user_data")?.as_bytes()?;
let user_data_json: UserDataPublicKey = serde_json_wasm::from_str(
    std::str::from_utf8(&user_data)?
)?;
let pubkey_sec1 = BASE64_STANDARD.decode(&user_data_json.data)?;
```

## Verification (Future Strict Mode)

Full verification requires:

1. **COSE Signature Verification**
   - Extract the P-384 public key from `certificate`
   - Verify the COSE_Sign1 signature over the payload

2. **Certificate Chain Verification**
   - Verify `certificate` is signed by `cabundle[0]`
   - Verify chain up to root
   - Check all certificates are valid (not expired, not revoked)

3. **Root Trust**
   - Verify final certificate in chain is signed by embedded AWS Root CA
   - Or verify final certificate IS the embedded AWS Root CA

## no_std Compatibility

The parser is designed to work in `no_std` environments:

```toml
[dependencies]
coset = { version = "0.3", default-features = false }
ciborium = { version = "0.2", default-features = false }
```

For WASM contract compilation, heavy crypto operations (X.509 verification) are stubbed out, with full verification performed off-chain by the CLI.

## Security Considerations

1. **PCR Stability**: PCR values change when the enclave code changes. Monitor for unexpected changes.

2. **Nonce Usage**: For replay protection, include a unique nonce in attestation requests.

3. **Timestamp Validation**: The attestation document includes a timestamp; verify it's recent.

4. **Certificate Expiry**: Enclave certificates are short-lived; don't cache for too long.

5. **Debug Mode**: PCR 0 differs between debug and production builds; only allow production PCRs in config.
