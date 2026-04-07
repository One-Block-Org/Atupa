use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Profile a transaction from a JSON-RPC endpoint
    Profile {
        /// The transaction hash
        #[arg(short, long)]
        tx: String,

        /// RPC endpoint URL
        #[arg(short, long, default_value = "http://localhost:8545")]
        rpc: String,
    },
    /// Compare two transaction traces
    Diff {
        /// Base transaction hash
        #[arg(short, long)]
        base: String,

        /// Target transaction hash
        #[arg(short, long)]
        target: String,
    },
}

use ethos_rpc::EthClient;
use ethos_parser::{Parser as EthosParser, aggregator::Aggregator};
use ethos_output::SvgGenerator;
use ethos_core::TraceStep;
use std::fs;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    eprintln!("{}", "Ethos: High-Fidelity Ethereum Tracing Suite".bold().cyan());

    match &cli.command {
        Commands::Profile { tx, rpc } => {
            eprintln!("Profiling transaction: {} on {}", tx.green(), rpc.yellow());
            
            // Initialize elegant CLI spinner
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                    .template("{spinner:.cyan} {msg}")?,
            );
            spinner.enable_steady_tick(Duration::from_millis(100));

            let steps = if tx == "demo" {
                spinner.set_message("Generating offline demo trace... [1/2]");
                vec![
                    TraceStep { pc: 0, op: "PUSH1".into(), gas: 1000, gas_cost: 3, depth: 1, stack: None, memory: None },
                    TraceStep { pc: 1, op: "CALL".into(), gas: 997, gas_cost: 0, depth: 1, stack: None, memory: None },
                    TraceStep { pc: 0, op: "SLOAD".into(), gas: 500, gas_cost: 2100, depth: 2, stack: None, memory: None },
                    TraceStep { pc: 1, op: "SSTORE".into(), gas: 480, gas_cost: 20000, depth: 2, stack: None, memory: None },
                    TraceStep { pc: 2, op: "RETURN".into(), gas: 400, gas_cost: 0, depth: 2, stack: None, memory: None },
                    TraceStep { pc: 2, op: "STOP".into(), gas: 300, gas_cost: 0, depth: 1, stack: None, memory: None },
                ]
            } else {
                // 1. Fetch
                let client = EthClient::new(rpc.to_string());
                spinner.set_message("Fetching trace from node... [1/4]");
                
                let trace_res = match client.get_transaction_trace(tx).await {
                    Ok(res) => res,
                    Err(e) => {
                        spinner.finish_and_clear();
                        eprintln!("\n{} Could not fetch trace from node.", "Error:".bold().red());
                        eprintln!("{} Is your node running at {}?", "Hint:".cyan(), rpc.yellow().bold());
                        eprintln!("{} {}", "Details:".dimmed(), e);
                        std::process::exit(1);
                    }
                };
                
                // 2. Parse
                spinner.set_message(format!("Normalizing {} structLogs... [2/4]", trace_res.struct_logs.len()));
                EthosParser::normalize(trace_res.struct_logs)
            };
            
            // 3. Aggregate
            let aggregate_step_msg = if tx == "demo" { "[2/2]" } else { "[3/4]" };
            spinner.set_message(format!("Aggregating execution metrics... {}", aggregate_step_msg));
            let stacks = Aggregator::build_collapsed_stacks(&steps);
            
            // 4. Output
            let output_step_msg = if tx == "demo" { "[Done]" } else { "[4/4]" };
            spinner.set_message(format!("Generating visual flamegraph... {}", output_step_msg));
            let svg = SvgGenerator::generate_flamegraph(&stacks)?;
            
            let out_file = format!("profile_{}.svg", tx);
            fs::write(&out_file, svg)?;
            
            spinner.finish_with_message(format!("{} Profile saved to {}", "Success!".bold().green(), out_file.bold()));
        }
        Commands::Diff { base, target } => {
            eprintln!("Comparing traces: {} and {}", base.green(), target.yellow());
        }
    }

    Ok(())
}
