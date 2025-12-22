# Build Instructions

## Prerequisites

The following tools must be installed before building the contracts:

### 1. cargo-odra

Odra framework CLI for building smart contracts.

```bash
cargo install cargo-odra
```

### 2. wasm-opt

WebAssembly optimizer (optimizes WASM file size).

```bash
cargo install wasm-opt
```

### 3. wabt (WebAssembly Binary Toolkit)

Contains `wasm-strip` for stripping debug symbols from WASM files.

```bash
sudo apt install wabt
```

## Building Contracts

Once prerequisites are installed, build the contracts with:

```bash
cargo odra build
```

This will generate optimized WASM files in:
- `wasm/` (root directory)
- `styks-contracts/wasm/` (contracts directory)

## Running Tests

```bash
just test
```

Or directly:

```bash
cargo odra test -b casper
```

## Other Commands

See available commands:

```bash
just
```
