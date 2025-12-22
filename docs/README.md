# Styks Documentation

Documentation for the Styks Measurement Anchored Price Oracle.

For a high-level overview of the Styks system, see the [main project README](../README.md).

## Contents

| Document | Description |
|----------|-------------|
| [Quick Start](./quickstart.md) | Get up and running quickly |
| [Measurement Anchored Model](./measurement-anchored-model.md) | Architecture and design |
| [API Reference](./api-reference.md) | Contract entrypoint documentation |
| [Security Model](./security-model.md) | Trust assumptions and threat mitigations |

## Overview

Styks is a price oracle for the Casper blockchain that uses AWS Nitro Enclave attestation for verifiable off-chain data. The Measurement Anchored Model enables automatic key rotation by trusting enclave measurements (PCR values) rather than static public keys.

## Key Concepts

- **Measurement Anchored Trust**: Any key proven to originate from an enclave with allowlisted PCR measurements is automatically trusted
- **Signer Caching**: Verified signers are cached for fast-path price updates
- **Replay Protection**: Strict timestamp ordering prevents double-submission
- **Operational Controls**: Pause/unpause and signer revocation for incident response

## Related

- [Main README](../README.md) - Project overview, on-chain integration examples, system architecture
- [BUILD.md](../BUILD.md) - Build prerequisites and instructions
- [CLAUDE.md](../CLAUDE.md) - AI assistant instructions for this codebase
