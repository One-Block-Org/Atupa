pub mod aggregator;

use ethos_core::TraceStep;
use ethos_rpc::RawStructLog;

pub struct Parser;

impl Parser {
    /// Normalizes a raw Anvil/Geth structLog into our universal TraceStep schema.
    pub fn normalize(raw_logs: Vec<RawStructLog>) -> Vec<TraceStep> {
        raw_logs.into_iter().map(|log| {
            TraceStep {
                pc: log.pc,
                op: log.op,
                gas: log.gas,
                gas_cost: log.gas_cost,
                depth: log.depth,
                stack: log.stack,
                memory: log.memory,
            }
        }).collect()
    }
}
