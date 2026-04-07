use ethos_rpc::EthClient;
use ethos_parser::{Parser as EthosParser, aggregator::Aggregator};
use ethos_output::SvgGenerator;
use ethos_core::TraceStep;
use std::fs;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use colored::*;

use ethos_rpc::etherscan::EtherscanResolver;

/// Executes the profile command: fetching, parsing, aggregating and visualizing the trace.
pub async fn execute_profile(tx: &str, rpc: &str, is_demo: bool, out: Option<String>, etherscan_key: Option<String>) -> anyhow::Result<()> {
    let spinner = initialize_spinner()?;

    // 1. Fetch
    let steps = if is_demo {
        get_demo_trace(&spinner)
    } else {
        fetch_live_trace(tx, rpc, &spinner).await
    };

    // 2. Aggregate
    let aggregate_step_msg = if is_demo { "[2/2]" } else { "[3/4]" };
    spinner.set_message(format!("Aggregating execution metrics... {}", aggregate_step_msg));
    let mut stacks = Aggregator::build_collapsed_stacks(&steps);

    // 3. Etherscan Resolution
    spinner.set_message("Resolving Contract Alignments against Etherscan...");
    let resolver = EtherscanResolver::new(etherscan_key);
    
    // We iterate sequentially to respect free-tier rate limits implicitly
    for stack in &mut stacks {
        if let Some(addr) = &stack.target_address {
             if let Some(name) = resolver.resolve_contract_name(addr).await {
                 stack.target_address = Some(name);
             }
        }
    }

    // 4. Output
    render_and_save_trace(stacks, tx, is_demo, out, &spinner)?;

    Ok(())
}

fn initialize_spinner() -> anyhow::Result<ProgressBar> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.cyan} {msg}")?,
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    Ok(spinner)
}

fn get_demo_trace(spinner: &ProgressBar) -> Vec<TraceStep> {
    spinner.set_message("Generating offline demo trace... [1/2]");
    vec![
        TraceStep { pc: 0, op: "PUSH1".into(), gas: 1000, gas_cost: 3, depth: 1, stack: None, memory: None, error: None, reverted: false },
        // Instead of calling, let's simulate a deep call
        TraceStep { pc: 1, op: "CALL".into(), gas: 997, gas_cost: 2600, depth: 1, stack: Some(vec![
            "0x0000000000000000000000000000000000000000".into(), // ret length
            "0x0000000000000000000000000000000000000000".into(), // ret offset
            "0x0000000000000000000000000000000000000000".into(), // args length
            "0x0000000000000000000000000000000000000100".into(), // args offset
            "0x0000000000000000000000000000000000000000".into(), // value
            "0x000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".into(), // USDC address!
            "0x10000".into() // gas
        ]), memory: None, error: None, reverted: false },
        TraceStep { pc: 0, op: "SLOAD".into(), gas: 500, gas_cost: 2100, depth: 2, stack: None, memory: None, error: None, reverted: false },
        TraceStep { pc: 1, op: "SSTORE".into(), gas: 480, gas_cost: 20000, depth: 2, stack: None, memory: None, error: None, reverted: false },
        TraceStep { pc: 2, op: "REVERT".into(), gas: 400, gas_cost: 5000, depth: 2, stack: None, memory: None, error: None, reverted: true }, // FAILS
        TraceStep { pc: 2, op: "STOP".into(), gas: 300, gas_cost: 0, depth: 1, stack: None, memory: None, error: None, reverted: false },
    ]
}

async fn fetch_live_trace(tx: &str, rpc: &str, spinner: &ProgressBar) -> Vec<TraceStep> {
    spinner.set_message("Connecting to EVM Node via JSON-RPC... [1/4]");
    let client = EthClient::new(rpc.to_string());
    
    let trace_res = match tokio::time::timeout(Duration::from_secs(15), client.get_transaction_trace(tx)).await {
        Ok(Ok(res)) => res,
        Ok(Err(e)) => {
            spinner.finish_and_clear();
            eprintln!("\n{} Could not fetch trace from node.", "Error:".bold().red());
            eprintln!("{} Verify your RPC node is running at {}?", "Hint:".cyan(), rpc.yellow().bold());
            eprintln!("{} {}", "Details:".dimmed(), e);
            std::process::exit(1);
        }
        Err(_) => {
            spinner.finish_and_clear();
            eprintln!("\n{} Connection to RPC node timed out after 15 seconds.", "Timeout:".bold().red());
            eprintln!("{} Is the node fully synced and responding at {}?", "Hint:".cyan(), rpc.yellow().bold());
            std::process::exit(1);
        }
    };
    
    spinner.set_message(format!("Normalizing {} structLogs... [2/4]", trace_res.struct_logs.len()));
    EthosParser::normalize(trace_res.struct_logs)
}

fn render_and_save_trace(
    stacks: Vec<ethos_core::CollapsedStack>,
    tx: &str,
    is_demo: bool,
    out: Option<String>,
    spinner: &ProgressBar,
) -> anyhow::Result<()> {
    // 4. Render
    let output_step_msg = if is_demo { "[Done]" } else { "[4/4]" };
    spinner.set_message(format!("Generating visual flamegraph... {}", output_step_msg));
    let svg = SvgGenerator::generate_flamegraph(&stacks)?;
    
    // 5. Output
    let out_file = out.unwrap_or_else(|| format!("profile_{}.svg", if is_demo { "demo" } else { tx }));
    fs::write(&out_file, svg)?;
    
    spinner.finish_with_message(format!("{} Profile saved to {}", "Success!".bold().green(), out_file.bold()));
    
    Ok(())
}
