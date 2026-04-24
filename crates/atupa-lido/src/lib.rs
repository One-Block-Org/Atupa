//! # atupa-lido — DeepTracer
//!
//! Lido stETH protocol adapter for the Atupa EVM profiling engine.
//! Provides tracing capabilities for Liquid Staking Mechanics,
//! tracking gas usage across submitting ETH, sharing rebases,
//! and handling withdrawals.

use atupa_adapters::ProtocolAdapter;
use atupa_core::TraceStep;

/// Selectors for major Lido protocol operations.
const LIDO_SELECTORS: &[(&str, &str)] = &[
    ("0xa1903eab", "submit"),             // stETH.submit(address _referral) — CORRECT
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

// ---------------------------------------------------------------------------
// Protocol Adapter Implementation
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct LidoAdapter;

impl ProtocolAdapter for LidoAdapter {
    fn name(&self) -> &str {
        "Lido stETH"
    }

    fn resolve_label(&self, _address: Option<&str>, selector: Option<&str>) -> Option<String> {
        let sel = selector?;
        for &(known_sel, label) in LIDO_SELECTORS {
            if sel == known_sel {
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
            if selector == known_sel {
                return Some(format!("stETH::{label}"));
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Deep Tracer Implementation
// ---------------------------------------------------------------------------

pub struct LidoReport {
    pub total_gas: u64,
    pub staking_gas: u64,
    pub shares_transfers: usize,
    pub token_transfers: usize,
    pub oracle_updates: usize,
    pub wrapped_txs: usize,
    pub max_depth: u16,
    pub reverted: bool,
    pub labeled_calls: Vec<LabeledCall>,
}

pub struct LabeledCall {
    pub depth: u16,
    pub label: String,
    pub gas_cost: u64,
}

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

    pub fn analyze_staking(
        &self,
        _tx_hash: &str,
        steps: &[TraceStep],
    ) -> anyhow::Result<LidoReport> {
        let mut total_gas = 0u64;
        let mut staking_gas = 0u64;
        let mut shares_transfers = 0usize;
        let mut token_transfers = 0usize;
        let mut oracle_updates = 0usize;
        let mut wrapped_txs = 0usize;
        let mut max_depth = 0u16;
        let mut reverted = false;
        let mut labeled_calls = Vec::new();

        for step in steps {
            total_gas = total_gas.saturating_add(step.gas_cost);
            max_depth = max_depth.max(step.depth);
            if step.reverted {
                reverted = true;
            }

            // Robust detection: Look for CALL/STATICCALL and check the stack for selectors
            if step.op == "CALL" || step.op == "STATICCALL" || step.op == "DELEGATECALL" {
                if let Some(stack_vec) = step.stack.as_ref() {
                    // The selector is typically the last item on the stack during a call dispatch
                    if let Some(val_str) = stack_vec.last() {
                        let trimmed = val_str.trim_start_matches("0x");
                        if let Ok(val) = u64::from_str_radix(trimmed, 16) {
                            let sel_str = format!("0x{:08x}", val as u32);

                            if let Some(label) = self.adapter.resolve_label(None, Some(&sel_str)) {
                                if sel_str == "0xa1903eab" {
                                    staking_gas = staking_gas.saturating_add(100_000); // Base staking cost estimate
                                } else if sel_str == "0x39ba163b" {
                                    shares_transfers += 1;
                                } else if sel_str == "0xa9059cbb" {
                                    token_transfers += 1;
                                } else if sel_str == "0x8b6ca260" {
                                    oracle_updates += 1;
                                } else if sel_str == "0x0a19ea81" || sel_str == "0x1dfab2e1" {
                                    wrapped_txs += 1;
                                }

                                labeled_calls.push(LabeledCall {
                                    depth: step.depth,
                                    label,
                                    gas_cost: step.gas_cost,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Clean up sequential duplicate PUSH4/CALL inferences
        labeled_calls.dedup_by(|a, b| a.label == b.label && a.depth == b.depth);

        Ok(LidoReport {
            total_gas,
            staking_gas,
            shares_transfers,
            token_transfers,
            oracle_updates,
            wrapped_txs,
            max_depth,
            reverted,
            labeled_calls,
        })
    }
}
