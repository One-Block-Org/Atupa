# Atupa SDK

The official developer library for the [Atupa](https://github.com/One-Block-Org/Atupa) tracing suite. This SDK bundles high-performance EVM trace aggregation with specialized protocol adapters for DeFi auditing.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
atupa-sdk = "0.1.0"
```

## Usage

The SDK provides direct access to the `LidoDeepTracer` and `AaveDeepTracer` for programmatic transaction analysis.

```rust
use atupa::lido::LidoDeepTracer;
use atupa_core::ProtocolAdapter;

async fn analyze_lido_tx(trace: Vec<EVMFrame>) {
    let mut tracer = LidoDeepTracer::new();
    
    // Process trace frames
    for frame in trace {
        tracer.on_step(&frame);
    }
    
    // Generate the summary report
    let report = tracer.generate_report();
    log::info!("Total Staking Gas: {}", report.staking_gas);
}
```

## Features

- **`lido`**: (Included by default) Deep traces for stETH and wstETH.
- **`aave`**: (Included by default) Deep traces for Aave v3 and GHO.
- **`nitro`**: Unified EVM/Stylus tracing for Arbitrum.

## License

Atupa SDK is licensed under the MIT License and the Apache License, Version 2.0.
