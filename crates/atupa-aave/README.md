# atupa-aave — DeepTracer

**Aave v3 & GHO Protocol Adapter for the Atupa EVM Profiling Engine**

> Part of **One Block's** "Transparency Layer" — the DeepTracer suite.

`atupa-aave` is a specialized protocol adapter crate that plugs into the Atupa engine to provide deep, human-readable profiling of Aave v3 Pool operations, liquidation flows, and GHO stablecoin supply mechanics.

## Architecture

```
atupa-core (the engine)
    │
    └── atupa-aave (this crate)
            ├── AaveV3Adapter      — function selector → label resolution
            ├── LiquidationReport  — structured liquidation flow analysis
            └── GhoSupplyMetrics   — GHO mint/burn & supply risk tracking
```

## Capabilities

- Full `AaveV3Pool` function selector resolution (supply, borrow, repay, liquidate, flash loans)
- `liquidationCall` deep trace: collateral seized, debt covered, health factor delta
- GHO Facilitator tracking: mint/burn flows and bucket capacity monitoring
- Rich `LiquidationReport` output compatible with Atupa's flamegraph pipeline

## Usage

```rust
use atupa_aave::AaveDeepTracer;

let tracer = AaveDeepTracer::default();
let report = tracer.analyze_trace(&trace_steps)?;
println!("{}", report.summary());
```
