# atupa

The user-facing command-line interface for the [Atupa](https://github.com/One-Block-Org/Atupa) Ethereum tracing suite.

## Usage

```bash
# Install
cargo install atupa

# Run a profile
atupa profile --tx <TX_HASH> --rpc <RPC_URL>

# Visualization
atupa profile --tx <TX_HASH> --out profile.svg
```

## Configuration

`atupa` leverages a layered configuration system. You can define your default RPC URL and Etherscan API Key in `atupa.toml` at the root of your project or through environment variables like `ATUPA_RPC_URL`.
