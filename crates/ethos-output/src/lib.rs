use ethos_core::CollapsedStack;
use std::fmt::Write;

pub struct SvgGenerator;

impl SvgGenerator {
    /// Generates a simple text-based "flamegraph" block for CLI, or raw SVG payload.
    /// In a full implementation, this uses a robust crate like `inferno`.
    /// For Milestone 1, we will generate a valid SVG visualization string matching Ethos Aesthetic.
    pub fn generate_flamegraph(stacks: &[CollapsedStack]) -> anyhow::Result<String> {
        let mut svg = String::new();
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 500\" style=\"background-color:#0d1117\">\n");
        svg.push_str("  <style>\n");
        svg.push_str("    .text { font-family: 'Inter', monospace; fill: #ffffff; font-size: 12px; }\n");
        svg.push_str("    .box { fill: url(#ethos-gradient); stroke: #1f2937; stroke-width: 1px; }\n");
        svg.push_str("  </style>\n");
        svg.push_str("  <defs>\n");
        svg.push_str("    <linearGradient id=\"ethos-gradient\" x1=\"0%\" y1=\"0%\" x2=\"0%\" y2=\"100%\">\n");
        svg.push_str("      <stop offset=\"0%\" stop-color=\"#0ea5e9\" />\n");
        svg.push_str("      <stop offset=\"100%\" stop-color=\"#4f46e5\" />\n");
        svg.push_str("    </linearGradient>\n");
        svg.push_str("  </defs>\n");

        // Normalize weights
        let total_weight: u64 = stacks.iter().map(|s| s.weight).sum();
        let max_width = 980.0;
        let mut current_y = 20.0;

        for stack in stacks {
            if stack.weight == 0 { continue; }
            let width = (stack.weight as f64 / total_weight as f64) * max_width;
            if width < 1.0 { continue; } // Too small to render

            // Format variables dynamically here where needed
            svg.push_str(&format!("  <rect x=\"10\" y=\"{}\" width=\"{}\" height=\"20\" class=\"box\" />\n", current_y, width));
            
            let leaf_name = stack.stack.split(';').last().unwrap_or("unknown");
            
            svg.push_str(&format!("  <text x=\"15\" y=\"{}\" class=\"text\">{} ({} gas)</text>\n", current_y + 14.0, leaf_name, stack.weight));
            
            current_y += 25.0;
        }

        svg.push_str("</svg>\n");
        Ok(svg)
    }
}
