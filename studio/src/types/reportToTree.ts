// ─── Atupa Studio — Report to Flamegraph Tree ────────────────────────────────
//
// Converts a flat `UnifiedStep[]` (with `depth` integers) into a tree of
// `FlameNode` objects suitable for hierarchical layout and d3-hierarchy.

import type { VmKind, StitchedReport } from './trace';

// ─── Tree Node ────────────────────────────────────────────────────────────────

export interface FlameNode {
  /** Unique identifier for stable React keys */
  id: string;
  name: string;
  vm: VmKind;
  /** Cumulative cost in gas-equiv for this node and all its children */
  value: number;
  /** Self cost only (no children) */
  selfCost: number;
  /** Step index in the original trace — useful for cross-linking */
  stepIndex: number;
  depth: number;
  is_vm_boundary: boolean;
  children: FlameNode[];
}

// ─── Conversion ───────────────────────────────────────────────────────────────

/**
 * Builds a single virtual root node whose children represent the call tree.
 *
 * Strategy:
 *  - Maintain a stack of "open" ancestors by depth.
 *  - When a step's depth is <= top of stack, pop until we find the correct parent.
 *  - Each step becomes a leaf under its parent; costs are aggregated up during
 *    a separate post-order pass.
 */
export function reportToTree(report: StitchedReport): FlameNode {
  const root: FlameNode = {
    id: 'root',
    name: report.tx_hash ? `tx ${report.tx_hash.slice(0, 8)}…` : 'Transaction',
    vm: 'Evm',
    value: 0,
    selfCost: 0,
    stepIndex: -1,
    depth: 0,
    is_vm_boundary: false,
    children: [],
  };

  if (report.steps.length === 0) return root;

  // Stack tracks the ancestral chain by [depth, node].
  // depth-0 slot is the virtual root.
  const stack: Array<{ depth: number; node: FlameNode }> = [
    { depth: 0, node: root },
  ];

  for (const step of report.steps) {
    const stepDepth = Math.max(1, step.depth); // never let depth be 0

    // Pop ancestors that are shallower-or-equal to current step
    while (stack.length > 1 && stack[stack.length - 1].depth >= stepDepth) {
      stack.pop();
    }

    const parent = stack[stack.length - 1].node;
    const selfCost = step.vm === 'Evm' ? step.gas_cost : step.cost_equiv;

    const node: FlameNode = {
      id: `step-${step.index}`,
      name: step.label,
      vm: step.vm,
      value: selfCost,   // will be aggregated post-order
      selfCost,
      stepIndex: step.index,
      depth: stepDepth,
      is_vm_boundary: step.is_vm_boundary,
      children: [],
    };

    parent.children.push(node);
    stack.push({ depth: stepDepth, node });
  }

  // Post-order aggregation: parent.value = Σ children.value + selfCost
  aggregateCosts(root);

  return root;
}

function aggregateCosts(node: FlameNode): number {
  if (node.children.length === 0) {
    node.value = node.selfCost;
    return node.value;
  }
  let childTotal = 0;
  for (const child of node.children) {
    childTotal += aggregateCosts(child);
  }
  node.value = node.selfCost + childTotal;
  return node.value;
}
