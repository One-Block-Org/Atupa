# Atupa Suit: System Architecture

The Atupa Suite is designed as a modular, high-performance infrastructure stack that provides transparency for the "Multi-VM" future of Ethereum. It separates the heavy lifting of raw trace parsing from the high-level business logic of protocol-specific auditing.

## 🏗 System Components

### 1. Network Layer (The Sources)
Atupa connects to diverse execution environments:
- **Ethereum (L1)**: Standard EVM via `structLogs`.
- **Arbitrum (L2)**: Dual-VM (EVM + Stylus) via the Nitro `stylusTracer`.
- **Unichain (L2)**: Real-time "Flashblocks" (200ms pending state).

### 2. Intelligence Layer (The Engine)
This is where raw hex data becomes human insight:
- **MixedTraceStitcher**: Correlates different trace formats (EVM, Stylus, Geth) into a unified timeline.
- **Protocol Adapters**: Specialized crates (`atupa-aave`, `atupa-lido`) that implement the `ProtocolAdapter` trait to extract domain-specific insights.
- **Symbol Resolver**: Uses DWARF symbols and Sourcify to map opcodes to source lines.

### 3. Interface Layer (The UX)
How developers and auditors interact with the data:
- **`atupa`**: The primary Rust binary that orchestrates parsing, auditing, and visualization.
- **`atupa-sdk`**: A high-level library that bundles the core engine and all protocol adapters for third-party integrations.
- **Atupa Report**: Automated, professional audit summaries generated directly from the terminal.

## 🏮 Data Formats
We use a unified **Atupa Profile JSON** that includes:
- `execution_steps`: Contiguous list of all VM instructions.
- `memory_deltas`: Mapping of memory growth and spikes.
- `protocol_context`: High-level labels and risk flags injected by Protocol Adapters (e.g., "Liquid Staking Share Rebase Detected").

---
🏮 *One Block: The Transparency Layer for the Hybrid Future.*
