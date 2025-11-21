use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "scalpel",
    version,
    about = "Surgical precision for Linux kernel observability and modification",
    long_about = r#"
╔═══════════════════════════════════════════════════════════╗
║  SCALPEL v1.0.0 - Kernel Surgeon Edition                 ║
║  Surgical Precision for the Linux Kernel                  ║
╚═══════════════════════════════════════════════════════════╝

Scalpel brings dynamic eBPF-based instrumentation to the Linux
kernel with natural language understanding. No recompilation,
safety first, semantic addressing.

EXAMPLES:
  scalpel                           # Launch interactive REPL
  scalpel analyze "trace tcp_v4_rcv where it drops packets"
  scalpel list                      # List active probes
  scalpel detach <probe-id>         # Detach a probe
"#
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch interactive REPL with TUI (default)
    Interactive,

    /// Analyze a prompt and show the operation plan
    Analyze {
        /// Natural language prompt describing the operation
        prompt: String,
    },

    /// List all active probes
    List,

    /// Detach a specific probe by ID
    Detach {
        /// Probe ID to detach
        probe_id: String,
    },

    /// Configuration management
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
}
