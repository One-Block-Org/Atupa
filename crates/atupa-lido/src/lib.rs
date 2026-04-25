//! # atupa-lido — DeepTracer
//!
//! Lido stETH protocol adapter for the Atupa EVM profiling engine.
//! Provides tracing capabilities for Liquid Staking Mechanics,
//! tracking gas usage across submitting ETH, sharing rebases,
//! and handling withdrawals.

use atupa_adapters::ProtocolAdapter;
use atupa_core::{DiffRow, ProtocolDiffReport, TraceStep};
use serde::{Deserialize, Serialize};

/// Selectors for major Lido protocol operations.
const LIDO_SELECTORS: &[(&str, &str)] = &[
    ("0xa1903eab", "submit"),             // stETH.submit(address _referral)
    ("0xea598cb0", "requestWithdrawals"), // Legacy request withdrawals
    ("0x826a73d6", "requestWithdrawalsWithPermit"),
    ("0xe35ea9a5", "claimWithdrawals"),
    ("0x8b6ca260", "handleOracleReport"), // Rebase oracle consensus
    ("0x39ba163b", "transferShares"),
    ("0x4dbcaef1", "transferSharesFrom"),
    ("0xa9059cbb", "transfer"), // ERC-20 generic
    ("0x095ea7b3", "approve"),  // ERC-20 generic
    ("0x0a19ea81", "wrap"),     // wstETH wrap
    ("0x1dfab2e1", "unwrap"),   // wstETH unwrap
];

/// Known Lido protocol contract addresses (Mainnet).
const LIDO_ADDRESSES: &[(&str, &str)] = &[
    (
        "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84",
        "stETH (Lido Core)",
    ),
    (
        "0x55032650b14df07b85bF18A3a3eC8E0Af2e028d5",
        "NodeOperatorsRegistry",
    ),
    ("0x442af752419395f27ed54A848524a30028962bb2", "LidoOracle"),
    (
        "0x889edC2Bf57978ed079b851D273218ee42a2b349",
        "WithdrawalQueue",
    ),
    ("0x852f970761d74367f33B6C2e309a29D681E2F16a", "LegacyOracle"),
    ("0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", "wstETH"),
];

// ---------------------------------------------------------------------------
// Protocol Adapter Implementation
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct LidoAdapter;

impl ProtocolAdapter for LidoAdapter {
    fn name(&self) -> &str {
        "Lido stETH"
    }

    fn resolve_label(&self, address: Option<&str>, selector: Option<&str>) -> Option<String> {
        if let Some(addr) = address {
            for &(known_addr, name) in LIDO_ADDRESSES {
                if addr.to_lowercase() == known_addr.to_lowercase() {
                    return Some(format!("Lido::{}", name));
                }
            }
        }

        let sel = selector?;
        for &(known_sel, label) in LIDO_SELECTORS {
            if sel.contains(known_sel.trim_start_matches("0x")) {
                return Some(format!("stETH::{label}"));
            }
        }
        None
    }
}

impl LidoAdapter {
    /// Resolve a 4-byte selector string to a human-readable label (no instance needed).
    pub fn resolve_selector_label(selector: &str) -> Option<String> {
        for &(known_sel, label) in LIDO_SELECTORS {
            if selector.contains(known_sel.trim_start_matches("0x")) {
                return Some(format!("stETH::{label}"));
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Report Structures
// ---------------------------------------------------------------------------

/// Detailed metrics for a Lido protocol interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidoReport {
    pub tx_hash: String,
    pub total_gas: u64,
    pub storage_reads: u32,
    pub storage_writes: u32,
    pub external_calls: u32,
    pub shares_transfers: u32,
    pub oracle_reports: u32,
    pub withdrawal_requests: u32,
    pub withdrawal_claims: u32,
    pub wrapped_ops: u32,
    pub max_depth: u16,
    pub reverted: bool,
    pub labeled_calls: Vec<LabeledCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledCall {
    pub depth: u16,
    pub label: String,
    pub gas_cost: u64,
}

// ---------------------------------------------------------------------------
// Deep Tracer Implementation
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct LidoDeepTracer {
    adapter: LidoAdapter,
}

impl LidoDeepTracer {
    pub fn new() -> Self {
        Self {
            adapter: LidoAdapter,
        }
    }

    /// Analyze a sequence of trace steps for Lido-specific patterns.
    pub fn analyze_staking(
        &self,
        tx_hash: &str,
        steps: &[TraceStep],
    ) -> anyhow::Result<LidoReport> {
        let mut total_gas = 0u64;
        let mut storage_reads = 0u32;
        let mut storage_writes = 0u32;
        let mut external_calls = 0u32;
        let mut shares_transfers = 0u32;
        let mut oracle_reports = 0u32;
        let mut withdrawal_requests = 0u32;
        let mut withdrawal_claims = 0u32;
        let mut wrapped_ops = 0u32;
        let mut max_depth = 0u16;
        let mut labeled_calls = Vec::new();

        for step in steps {
            total_gas = total_gas.saturating_add(step.gas_cost);
            max_depth = max_depth.max(step.depth);

            match step.op.as_str() {
                "SLOAD" => storage_reads += 1,
                "SSTORE" => storage_writes += 1,
                "CALL" | "STATICCALL" | "DELEGATECALL" | "CALLCODE" => {
                    external_calls += 1;

                    let selector = step
                        .stack
                        .as_ref()
                        .and_then(|s| s.last())
                        .map(|s| s.as_str());

                    if let Some(label) = self.adapter.resolve_label(None, selector) {
                        if label.contains("transferShares") {
                            shares_transfers += 1;
                        } else if label.contains("handleOracleReport") {
                            oracle_reports += 1;
                        } else if label.contains("requestWithdrawals") {
                            withdrawal_requests += 1;
                        } else if label.contains("claimWithdrawals") {
                            withdrawal_claims += 1;
                        } else if label.contains("wrap") || label.contains("unwrap") {
                            wrapped_ops += 1;
                        }

                        labeled_calls.push(LabeledCall {
                            depth: step.depth,
                            label,
                            gas_cost: step.gas_cost,
                        });
                    }
                }
                _ => {}
            }
        }

        let reverted = steps.last().is_some_and(|s| s.reverted);
        labeled_calls.dedup_by(|a, b| a.label == b.label && a.depth == b.depth);

        Ok(LidoReport {
            tx_hash: tx_hash.to_string(),
            total_gas,
            storage_reads,
            storage_writes,
            external_calls,
            shares_transfers,
            oracle_reports,
            withdrawal_requests,
            withdrawal_claims,
            wrapped_ops,
            max_depth,
            reverted,
            labeled_calls,
        })
    }

    /// Perform a deep field-by-field diff between two Lido executions.
    pub fn diff_reports(
        &self,
        base_tx: &str,
        base_steps: &[TraceStep],
        target_tx: &str,
        target_steps: &[TraceStep],
    ) -> anyhow::Result<ProtocolDiffReport> {
        let base = self.analyze_staking(base_tx, base_steps)?;
        let target = self.analyze_staking(target_tx, target_steps)?;

        let rows = vec![
            DiffRow::new(
                "Total Gas",
                base.total_gas as f64,
                target.total_gas as f64,
                true,
            ),
            DiffRow::new(
                "Storage Reads",
                base.storage_reads as f64,
                target.storage_reads as f64,
                true,
            ),
            DiffRow::new(
                "Storage Writes",
                base.storage_writes as f64,
                target.storage_writes as f64,
                true,
            ),
            DiffRow::new(
                "External Calls",
                base.external_calls as f64,
                target.external_calls as f64,
                true,
            ),
            DiffRow::new(
                "Shares Transfers",
                base.shares_transfers as f64,
                target.shares_transfers as f64,
                true,
            ),
            DiffRow::new(
                "Oracle Reports",
                base.oracle_reports as f64,
                target.oracle_reports as f64,
                true,
            ),
            DiffRow::new(
                "Withdrawal Requests",
                base.withdrawal_requests as f64,
                target.withdrawal_requests as f64,
                true,
            ),
            DiffRow::new(
                "Withdrawal Claims",
                base.withdrawal_claims as f64,
                target.withdrawal_claims as f64,
                true,
            ),
            DiffRow::new(
                "Wrapped Ops",
                base.wrapped_ops as f64,
                target.wrapped_ops as f64,
                true,
            ),
        ];

        Ok(ProtocolDiffReport {
            protocol: "Lido stETH".to_string(),
            rows,
        })
    }
}
