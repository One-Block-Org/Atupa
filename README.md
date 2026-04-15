<p align="center">
  <img src="assets/logo.png" width="350" alt="Atupa Logo">
</p>

<h1 align="center">Atupa</h1>

<p align="center">
  <strong>High-Fidelity Ethereum Tracing, Profiling &amp; Visual Analysis Suite</strong>
</p>

<p align="center">
  <a href="https://github.com/One-Block-Org/Atupa/actions"><img src="https://github.com/One-Block-Org/Atupa/actions/workflows/rust.yml/badge.svg" alt="CI Status"></a>
  <a href="https://crates.io/crates/atupa-core"><img src="https://img.shields.io/crates/v/atupa-core.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/atupa-core"><img src="https://docs.rs/atupa-core/badge.svg" alt="Documentation"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg" alt="License"></a>
</p>

---

**Atupa** (meaning *Lantern/Lamp*) is a professional-grade EVM + Arbitrum Stylus execution profiler. It turns raw JSON-RPC `debug_traceTransaction` and `stylusTracer` logs into actionable visual insights — from gas flamegraphs to unified EVM/WASM execution dashboards.

## ✨ Key Features

- **🔥 Unified EVM + Stylus Tracing**: Stitches data from both the EVM and Stylus WASM runtime into a single coherent execution timeline.
- **🏮 Atupa Studio**: A local-first web visualizer — drop a `report.json` to instantly render metric cards, HostIO hot paths, and a step-by-step trace inspector.
- **📊 HostIO Flamegraph**: Surfaces the most expensive Stylus Host I/O calls (`storage_flush_cache`, `native_keccak256`, etc.) ranked by gas-equivalent cost.
- **🚨 Crisp Revert Identification**: Instantly identifies failing sub-calls with high-contrast highlights.
- **🔍 Smart Contract Resolution**: Automatically resolves hex addresses to verified contract names via Etherscan V2.
- **💉 Protocol-Specific Deep Auditing**: Built-in deep traces for **Lido stETH** and **Aave v3**.
- **🛠 Modular Library Architecture**: Pure Rust workspace with specialized crates for adapters, RPC, parsing, and output.

## 🚀 Quick Start

### Installation

```bash
cargo install atupa
```

### Capturing a Unified Trace

```bash
# Capture an Arbitrum Stylus transaction (summary to terminal)
atupa capture --tx 0x... --rpc https://arb-mainnet.g.alchemy.com/v2/KEY

# Export as JSON for Atupa Studio
atupa capture --tx 0x... --rpc https://... --output json --file report.json

# Deep protocol audit (Lido or Aave)
atupa audit --protocol lido --tx 0x...

# Compare execution cost of two transactions
atupa diff --base 0x... --target 0x...
```

### Run the Demo
```bash
atupa profile --demo
```

## 🏮 Atupa Studio

Atupa Studio is a local-first web visualizer for your execution traces.

```bash
# Start the Studio dev server
cd studio && npm install && npm run dev
```

Then open **http://localhost:5173**, generate a trace with `--output json --file report.json`, and drop the file into the Studio. The dashboard instantly renders:

- **Execution Metrics** — EVM Gas, Stylus Ink, HostIO call counts, VM boundary crossings
- **HostIO Hot Paths** — Ranked table with inline distribution bars
- **Trace Inspector** — Paginated, filterable, searchable step-by-step execution viewer

## 📦 Project Structure

Atupa is built as a highly modular monorepo:

| Crate / Directory | Description |
|---|---|
| [`bin/atupa`](bin/atupa) | The primary command-line interface. |
| [`studio/`](studio) | Atupa Studio — Vite + React web visualizer. |
| [`crates/atupa-sdk`](crates/atupa-sdk) | Public-facing SDK for programmatic tracing. |
| [`crates/atupa-core`](crates/atupa-core) | Shared types and core configuration logic. |
| [`crates/atupa-parser`](crates/atupa-parser) | Aggregation engine that collapses EVM traces. |
| [`crates/atupa-nitro`](crates/atupa-nitro) | Arbitrum Nitro dual-VM stitcher (EVM + Stylus). |
| [`crates/atupa-rpc`](crates/atupa-rpc) | Async Ethereum JSON-RPC client & Etherscan resolver. |
| [`crates/atupa-lido`](crates/atupa-lido) | Specialized adapter for Lido stETH. |
| [`crates/atupa-aave`](crates/atupa-aave) | Specialized adapter for Aave v3 & GHO. |

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for more details.

## 📄 License

Atupa is dual-licensed under the [MIT License](LICENSE-MIT) and the [Apache License, Version 2.0](LICENSE-APACHE).
You may use this software under either license, at your option.
