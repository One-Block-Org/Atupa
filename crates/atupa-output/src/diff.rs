use atupa_core::{CollapsedStack, VmKind};
use std::collections::HashMap;

struct DiffEntry {
    stack: String,
    depth: u16,
    vm_kind: VmKind,
    baseline_weight: u64,
    target_weight: u64,
    resolved_label: Option<String>,
    target_address: Option<String>,
    reverted: bool,
}

pub fn generate_diff_flamegraph(
    baseline_stacks: &[CollapsedStack],
    target_stacks: &[CollapsedStack],
) -> anyhow::Result<String> {
    // 1. Merge Stacks by exact path string
    let mut merged: HashMap<String, DiffEntry> = HashMap::new();

    for s in baseline_stacks {
        merged.insert(s.stack.clone(), DiffEntry {
            stack: s.stack.clone(),
            depth: s.depth,
            vm_kind: s.vm_kind.clone(),
            baseline_weight: s.weight,
            target_weight: 0,
            resolved_label: s.resolved_label.clone(),
            target_address: s.target_address.clone(),
            reverted: s.reverted,
        });
    }

    for s in target_stacks {
        if let Some(entry) = merged.get_mut(&s.stack) {
            entry.target_weight += s.weight;
        } else {
            merged.insert(s.stack.clone(), DiffEntry {
                stack: s.stack.clone(),
                depth: s.depth,
                vm_kind: s.vm_kind.clone(),
                baseline_weight: 0,
                target_weight: s.weight,
                resolved_label: s.resolved_label.clone(),
                target_address: s.target_address.clone(),
                reverted: s.reverted,
            });
        }
    }

    let entries: Vec<&DiffEntry> = merged.values().collect();
    if entries.is_empty() || entries.iter().all(|e| e.baseline_weight == 0 && e.target_weight == 0) {
        return Ok("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 60\" \
                   style=\"background-color:#0d1117\">\
                   <text x=\"14\" y=\"34\" fill=\"#94a3b8\" \
                   font-family=\"Inter,monospace\" font-size=\"13\">\
                   No execution data found for diff.\
                   </text></svg>".to_string());
    }

    const SVG_W: f64 = 1000.0;
    const PAD_L: f64 = 10.0;
    const CHART_W: f64 = SVG_W - PAD_L * 2.0;
    const BAR_H: f64 = 26.0;
    const GAP: f64 = 4.0;
    const HEADER_H: f64 = 60.0;
    const SEPARATOR_H: f64 = 28.0;
    const MIN_BAR_PX: f64 = 2.0;

    let mut evm_entries: Vec<&&DiffEntry> = entries.iter().filter(|e| e.vm_kind == VmKind::Evm).collect();
    let mut wasm_entries: Vec<&&DiffEntry> = entries.iter().filter(|e| e.vm_kind == VmKind::Stylus).collect();
    let has_wasm = !wasm_entries.is_empty();

    let mut depths: Vec<u16> = evm_entries.iter().map(|e| e.depth).collect();
    depths.sort_unstable();
    depths.dedup();

    let mut svg = String::new();
    // We will build the SVG body first to know the total height
    let mut body = String::new();
    let mut current_y = HEADER_H;

    // ── EVM lanes ─────────────────────────────────────────────────────────────
    for depth in &depths {
        let mut lane_entries: Vec<&&&DiffEntry> = evm_entries.iter().filter(|e| e.depth == *depth).collect();
        // Sort by stack string to maintain deterministic left-to-right ordering
        lane_entries.sort_by(|a, b| a.stack.cmp(&b.stack));

        let lane_weight: u64 = lane_entries.iter().map(|e| std::cmp::max(e.baseline_weight, e.target_weight)).sum();
        if lane_weight == 0 {
            continue;
        }

        let mut bar_x = PAD_L;
        for entry in &lane_entries {
            let node_weight = std::cmp::max(entry.baseline_weight, entry.target_weight);
            if node_weight == 0 {
                continue;
            }
            let bar_w = (node_weight as f64 / lane_weight as f64) * CHART_W;
            if bar_w < MIN_BAR_PX {
                continue;
            }

            render_diff_bar(&mut body, entry, bar_x, current_y, bar_w - 1.0, BAR_H);
            bar_x += bar_w;
        }
        current_y += BAR_H + GAP;
    }

    // ── WASM lanes ────────────────────────────────────────────────────────────
    if has_wasm {
        current_y += SEPARATOR_H;
        
        // Draw separator
        body.push_str(&format!(
            r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#334155" stroke-width="1" stroke-dasharray="4"/>"##,
            PAD_L, current_y - 14.0, SVG_W - PAD_L, current_y - 14.0
        ));
        body.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="11" fill="#64748b" font-family="Inter, monospace" text-anchor="middle" font-weight="bold">STYLUS HOST I/O</text>"##,
            SVG_W / 2.0, current_y - 10.0
        ));

        let global_wasm_weight: u64 = wasm_entries.iter().map(|e| std::cmp::max(e.baseline_weight, e.target_weight)).sum();
        wasm_entries.sort_by(|a, b| a.stack.cmp(&b.stack));

        let mut bar_x = PAD_L;
        for entry in &wasm_entries {
            let node_weight = std::cmp::max(entry.baseline_weight, entry.target_weight);
            if node_weight == 0 {
                continue;
            }
            let bar_w = if global_wasm_weight > 0 {
                (node_weight as f64 / global_wasm_weight as f64) * CHART_W
            } else {
                CHART_W / wasm_entries.len() as f64
            };
            if bar_w < MIN_BAR_PX {
                continue;
            }

            render_diff_bar(&mut body, entry, bar_x, current_y, bar_w - 1.0, BAR_H);
            bar_x += bar_w;
        }
        current_y += BAR_H + GAP;
    }

    let total_height = current_y + 60.0;

    // Build final SVG
    svg.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}" style="background-color:#0d1117">"##,
        SVG_W, total_height, SVG_W, total_height
    ));

    svg.push_str(
        r##"<style>
            .func { font: 12px Inter, monospace; transition: all 0.1s ease; } 
            .func:hover { stroke: #ffffff; stroke-width: 1.5; cursor: pointer; filter: brightness(1.2); }
            @keyframes slideIn { from { opacity: 0; transform: translateY(-5px); } to { opacity: 1; transform: translateY(0); } }
            rect { animation: slideIn 0.3s ease-out forwards; }
        </style>"##
    );

    // Title
    svg.push_str(&format!(
        r##"<text x="{}" y="30" font-size="18" fill="#e2e8f0" font-family="Inter, monospace" text-anchor="middle" font-weight="bold">Atupa Visual Diff Flamegraph</text>"##,
        SVG_W / 2.0
    ));

    // Legend
    render_diff_legend(&mut svg, total_height - 20.0);

    svg.push_str(&body);
    svg.push_str("</svg>");
    Ok(svg)
}

fn render_diff_bar(out: &mut String, entry: &DiffEntry, x: f64, y: f64, w: f64, h: f64) {
    let baseline = entry.baseline_weight;
    let target = entry.target_weight;
    
    let color = get_diff_color(baseline, target);
    let tooltip = format_diff_tooltip(entry);
    
    out.push_str(&format!(
        r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" stroke="#1e293b" stroke-width="1.0" class="func" rx="3">"##,
        x, y, w, h, color
    ));
    out.push_str(&format!(r##"<title>{}</title></rect>"##, tooltip));

    let display_name = get_truncated_name(&entry.stack, &entry.resolved_label, &entry.target_address, w);
    if !display_name.is_empty() {
        out.push_str(&format!(
            r##"<text x="{:.2}" y="{:.2}" dx="6" dy="17" font-size="12" fill="#f8fafc" font-family="Inter, monospace" style="pointer-events:none">{}</text>"##,
            x, y, display_name
        ));
    }
}

fn get_diff_color(baseline: u64, target: u64) -> String {
    if baseline == 0 && target == 0 {
        return "#334155".into();
    }
    if baseline == 0 {
        return "#ef4444".into(); // Regression (Red)
    } 
    if target == 0 {
        return "#22c55e".into(); // Improvement (Green)
    } 

    let change = (target as f64 - baseline as f64) / baseline as f64;

    if change > 0.01 {
        let intensity = ((change * 100.0).min(100.0) / 100.0) * 0.6; 
        format!("rgba(239, 68, 68, {:.2})", 0.4 + intensity)
    } else if change < -0.01 {
        let intensity = ((change.abs() * 100.0).min(100.0) / 100.0) * 0.6;
        format!("rgba(34, 197, 94, {:.2})", 0.4 + intensity)
    } else {
        "#475569".into() // Stable (Slate)
    }
}

fn format_diff_tooltip(entry: &DiffEntry) -> String {
    let baseline = entry.baseline_weight;
    let target = entry.target_weight;
    let leaf = entry.stack.split(';').next_back().unwrap_or(&entry.stack);

    let prefix = if entry.reverted { "REVERTED — " } else { "" };
    let vm = if entry.vm_kind == VmKind::Evm { "EVM" } else { "Stylus" };

    if baseline == 0 {
        return format!("{}{} [{}] | NEW: {} gas", prefix, leaf, vm, target);
    }
    if target == 0 {
        return format!("{}{} [{}] | REMOVED: {} gas", prefix, leaf, vm, baseline);
    }

    let diff = target as i64 - baseline as i64;
    let percent = (diff as f64 / baseline as f64) * 100.0;

    format!(
        "{}{} [{}] | {} -> {} gas ({:+.2}%)",
        prefix, leaf, vm, baseline, target, percent
    )
}

fn get_truncated_name(stack: &str, resolved: &Option<String>, addr: &Option<String>, w: f64) -> String {
    let leaf = stack.split(';').next_back().unwrap_or(stack);
    let base = if let Some(r) = resolved {
        r.clone()
    } else if let Some(a) = addr {
        format!("{} [{}]", leaf, a)
    } else {
        leaf.to_string()
    };

    let max_chars = ((w - 12.0) / 7.5) as usize; 
    if max_chars < 3 {
        return String::new();
    }
    if base.len() <= max_chars {
        base
    } else {
        format!("{}…", &base[..max_chars.saturating_sub(1)])
    }
}

fn render_diff_legend(out: &mut String, y: f64) {
    let items = [
        ("Regression (Target &gt; Base)", "#ef4444"),
        ("Improvement (Target &lt; Base)", "#22c55e"),
        ("No Change", "#475569"),
    ];

    let start_x = (1000.0 - (items.len() as f64 * 200.0)) / 2.0;

    for (i, (label, color)) in items.iter().enumerate() {
        let x = start_x + (i as f64 * 240.0);
        out.push_str(&format!(
            r##"<rect x="{}" y="{}" width="16" height="16" fill="{}" rx="4"/>"##,
            x, y - 12.0, color
        ));
        out.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="13" fill="#cbd5e1" font-family="Inter, monospace">{}</text>"##,
            x + 24.0, y, label
        ));
    }
}
