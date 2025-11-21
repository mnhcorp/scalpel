mod cli;
mod config;
mod error;
mod kernel;
mod llm;
mod bpf;
mod surgical_loop;
mod tui;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scalpel=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command line arguments
    let args = cli::Args::parse();

    // Load configuration
    let config = config::Config::load()?;

    match args.command {
        Some(cli::Commands::Interactive) | None => {
            // Launch interactive REPL with TUI
            tui::run_interactive(config).await?;
        }
        Some(cli::Commands::Analyze { prompt }) => {
            // One-shot analysis mode
            let mut loop_engine = surgical_loop::SurgicalLoop::new(config);
            let result = loop_engine.analyze(&prompt).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Some(cli::Commands::List) => {
            // List active probes
            let probes = kernel::probe::list_active_probes()?;
            for probe in probes {
                println!("{}", probe);
            }
        }
        Some(cli::Commands::Detach { probe_id }) => {
            // Detach a specific probe
            kernel::probe::detach_probe(&probe_id)?;
            println!("Probe {} detached successfully", probe_id);
        }
        Some(cli::Commands::Config { show }) => {
            if show {
                println!("{}", config);
            }
        }
    }

    Ok(())
}
