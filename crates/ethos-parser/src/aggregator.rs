use ethos_core::{CollapsedStack, TraceStep};
use std::collections::HashMap;
use log::debug;

pub struct Aggregator;

impl Aggregator {
    /// Build collapsed stacks from a sequence of raw trace steps (structLogs style).
    ///
    /// # Algorithm
    /// 1. Walk through execution steps
    /// 2. Track call stack depth
    /// 3. Build stack strings for each gas-consuming operation
    /// 4. Aggregate by unique stack (sum gas weights)
    pub fn build_collapsed_stacks(steps: &[TraceStep]) -> Vec<CollapsedStack> {
        debug!("Building collapsed stacks from {} execution steps", steps.len());

        // Map to aggregate stacks: stack_string -> (total_gas, last_pc)
        let mut stack_map: HashMap<String, (u64, u64)> = HashMap::new();

        // Current call stack
        let mut call_stack: Vec<String> = Vec::new();

        for step in steps {
            let operation = step.op.clone();
            let current_depth = step.depth as usize;

            // If depth decreased, we returned from function calls
            if current_depth < call_stack.len() {
                call_stack.truncate(current_depth);
            }

            // If depth increased, we entered a new call
            while call_stack.len() < current_depth {
                call_stack.push("CALL".to_string());
            }

            // Build the full stack string with current operation
            let stack_str = if call_stack.is_empty() {
                operation.clone()
            } else {
                format!("{};{}", call_stack.join(";"), operation)
            };

            // Accumulate gas cost
            let entry = stack_map.entry(stack_str).or_insert((0, 0));
            entry.0 += step.gas_cost;
            entry.1 = step.pc;

            // Important: we push the actual smart contract address or function 
            // if we can extract it in the future, but for raw structural mapping, 
            // the operation often serves as the leaf node.
        }

        let mut stacks: Vec<CollapsedStack> = stack_map
            .into_iter()
            .map(|(stack, (weight, pc))| CollapsedStack {
                stack,
                weight,
                last_pc: Some(pc),
            })
            .collect();

        stacks.sort_by(|a, b| b.weight.cmp(&a.weight));
        debug!("Built {} unique collapsed stacks", stacks.len());

        stacks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethos_core::TraceStep;

    #[test]
    fn test_aggregator_collapses_simple_call() {
        let steps = vec![
            // Root context opcodes (Depth 1)
            TraceStep { pc: 0, op: "PUSH1".into(), gas: 100, gas_cost: 3, depth: 1, stack: None, memory: None },
            TraceStep { pc: 1, op: "CALL".into(), gas: 90, gas_cost: 0, depth: 1, stack: None, memory: None },
            // Sub-context opcodes (Depth 2)
            TraceStep { pc: 0, op: "SSTORE".into(), gas: 50, gas_cost: 20, depth: 2, stack: None, memory: None },
            TraceStep { pc: 1, op: "RETURN".into(), gas: 20, gas_cost: 0, depth: 2, stack: None, memory: None },
            // Back to root (Depth 1)
            TraceStep { pc: 2, op: "STOP".into(), gas: 15, gas_cost: 0, depth: 1, stack: None, memory: None },
        ];

        let stacks = Aggregator::build_collapsed_stacks(&steps);
        
        println!("Generated stacks: {:?}", stacks);
        
        assert!(!stacks.is_empty(), "Stacks should not be empty");
        
        let sstore_stack = stacks.iter().find(|s| s.stack == "CALL;CALL;SSTORE").expect("Should find SSTORE");
        assert_eq!(sstore_stack.weight, 20);

        let push1_stack = stacks.iter().find(|s| s.stack == "CALL;PUSH1").expect("Should find PUSH1");
        assert_eq!(push1_stack.weight, 3);
    }
}

