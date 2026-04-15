# Atupa Testing Guide

This document provides a comprehensive, step-by-step framework for testing the entirety of the Atupa project. It covers automated workspace tests, local CLI verification, and end-to-end integration flows.

---

## 1. Prerequisites & Environment Setup

Most advanced features of Atupa (Trace Capturing, Protocol Audits, Differential Profiling) require live Ethereum/Arbitrum node data. Ensure your environment is configured before starting E2E testing.

### Environment Variables
For live network tests, expose the following environment variables:
```bash
export ATUPA_RPC_URL="https://arb-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_KEY"
export ETHERSCAN_API_KEY="YOUR_ARBISCAN_KEY"
```
*(If you do not have an API key, you can still verify functionality using the `--demo` mode or cargo unit testing).*

---

## 2. Automated Workspace Testing (CI/CD Quality)

The project leverages Cargo's highly parallel test runner to ensure unit and integration validity across all workspace crates (`atupa-core`, `atupa-parser`, `atupa-lido`, `atupa-aave`, etc.).

**Run the fundamental verification suite:**
```bash
# 1. Formatting
cargo fmt --all -- --check

# 2. Linting
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. All Unit & Integration Tests
cargo test --workspace --all-features
```
> **Success Criteria:** Zero warnings and passing results for all doc tests, unit tests, and integration assertions.

---

## 3. CLI Feature Testing (Local Verification)

To test the CLI application manually and visually, compile it locally and execute its main subcommands. E2E testing evaluates the terminal presentation, progress bar animations, rendering, and API communication.

Compile first to ensure performance:
```bash
cargo build --release -p atupa
alias atupa="./target/release/atupa"
```

### Flow 1: Offline / Demo Verification
Verify the CLI profiling outputs and visual styles without relying on the network.
```bash
# Run a demo profile 
atupa profile --demo --tx 0x0
```
> **Expected Output:** You should see a visually distinct `eprintln!` profiling banner and a well-formatted statistical JSON structure showing unified traces.

### Flow 2: Live RPC Connection / Trace Capture
Test the `capture` command, evaluating both `stdout` output generation and diagnostic `stderr` headers. Replace `0x...` with a real target Arbitrum/Ethereum transaction.
```bash
atupa capture --tx 0x8a923...
```
> **Expected Output:** A cleanly rendered summary payload. (Verify that `atupa capture --tx ... > report.txt` results in the report being written cleanly to file without UI spinners overriding `stdout`).

### Flow 3: Specific Protocol Audits
Test the semantic decoding layers. These verify that the abstract trace data successfully parses into protocol-specific models.

**A. Test Aave v3 Decoding**
```bash
atupa audit --protocol aave --tx 0x93ab...
```
> **Expected Output:** A generated diagnostic table identifying the Flash Loan execution, debt updates, user reserve states, and liquidation values. 

**B. Test Lido stETH Execution**
```bash
atupa audit --protocol lido --tx 0x1fca...
```
> **Expected Output:** Verification of the staking pipeline (Deposit Event → Oracle Refresh → Staking Limits → Treasury Accounting).

### Flow 4: Differential Profiling (Cost & Execution Delta)
Verify the `--diff` execution flow, calculating the divergence in bytecode pathways or gas consumption between two similar transactions.

```bash
atupa diff 0xBASE_TX_HASH 0xTARGET_TX_HASH
```
> **Expected Output:** A dual-table view or structured report listing `-X%` or `+Y%` deltas for Execution Steps, Memory Allocation, EVM Gas, and System VM parameters.

---

## 4. Specific Crate Example Evaluation

To test individual programmatic SDK behaviors independent of the main CLI app, Atupa includes runnable examples inside specific crates.

### Run the Trace Parser Example
This runs a simulated test of the underlying log unification parser:
```bash
RUST_LOG=info cargo run -p atupa-parser --example trace_analysis
```
> **Expected Output:** The parsed log analysis will stream into the standard console output using `env_logger`. Ensure the output renders properly without utilizing `println!`.

---

## 5. Security & Safety

Atupa follows strict safety constraints. As part of your testing loop:
- Ensure the **"No-`println!`" rule** is respected. The CLI `stdout` must remain untainted for data payload piping, utilizing `eprintln!` strictly for diagnostics.
- Ensure all dependencies remain synchronized avoiding version drift across workspaces. Run `cargo update` locally and re-run step 2 (Automated Workspace Testing) to check against dependency vulnerabilities.
