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
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    println!("{}", "Ethos: High-Fidelity Ethereum Tracing Suite".bold().cyan());

    match &cli.command {
        Commands::Profile { tx, rpc } => {
            println!("Profiling transaction: {} on {}", tx.green(), rpc.yellow());
            
            // 1. Fetch
            let client = EthClient::new(rpc.to_string());
            println!("{} Fetching trace from node...", "[1/4]".bold().dimmed());
            let trace_res = client.get_transaction_trace(tx).await?;
            
            // 2. Parse
            println!("{} Normalizing {} structLogs...", "[2/4]".bold().dimmed(), trace_res.struct_logs.len());
            let steps = EthosParser::normalize(trace_res.struct_logs);
            
            // 3. Aggregate
            println!("{} Aggregating execution metrics...", "[3/4]".bold().dimmed());
            let stacks = Aggregator::build_collapsed_stacks(&steps);
            
            // 4. Output
            println!("{} Generating visual flamegraph...", "[4/4]".bold().dimmed());
            let svg = SvgGenerator::generate_flamegraph(&stacks)?;
            
            let out_file = format!("profile_{}.svg", tx);
            fs::write(&out_file, svg)?;
            
            println!("{} Profile saved to {}", "Success!".bold().green(), out_file.bold());
        }
        Commands::Diff { base, target } => {
            println!("Comparing traces: {} and {}", base.green(), target.yellow());
        }
    }

    Ok(())
}
