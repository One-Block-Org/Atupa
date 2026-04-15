# Atupa CLI

The primary command-line interface for the Atupa Ethereum tracing suite. `atupa` helps you visualize and audit complex EVM transactions directly from your terminal.

## Installation

```bash
cargo install atupa
```

## Commands

### `profile`
Generates a visual gas-weighted flamegraph for a transaction.

```bash
atupa profile --tx 0x... --rpc https://mainnet.infura.io/v3/YOUR_KEY
```

**Options:**
- `--tx <HASH>`: The transaction hash to profile.
- `--rpc <URL>`: The JSON-RPC endpoint (overrides `atupa.toml`).
- `--out <FILE>`: The output path for the SVG (default: `profile.svg`).
- `--demo`: Run an offline profile using a bundled trace log.

### `audit`
Performs a deep protocol-specific audit for supported DeFi protocols.

```bash
# Deep audit a Lido transaction
atupa audit --protocol lido --tx 0x...

# Deep audit an Aave v3 transaction
atupa audit --protocol aave --tx 0x...
```

**Supported Protocols:**
- `lido`: Tracks stETH/wstETH transfers, rebases, and staking gas.
- `aave`: Tracks v3 deposits, borrows, GHO actions, and liquidations.

## Configuration

`atupa` reads from an optional `atupa.toml` in your current directory:

```toml
rpc_url = "https://arbitrum-mainnet.infura.io/v3/..."
etherscan_key = "..."
```

## License

Licensed under the MIT License.
