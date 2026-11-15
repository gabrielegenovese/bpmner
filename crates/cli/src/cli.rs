use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Convert a BPMN choreography file to PNML and/or DOT
    Convert {
        /// Input BPMN file
        input: String,

        /// Output PNML
        #[arg(long)]
        pnml: Option<String>,

        /// Output DOT
        #[arg(long)]
        dot: Option<String>,

        /// Do not include the initial marking in the exported net
        #[arg(long)]
        no_initial_marking: bool,

        /// Lay out the DOT graph from left to right
        #[arg(long)]
        rankdir_lr: bool,
    },

    /// Analyze the encoded Petri net with TINA (safety and deadlock-freedom)
    Check {
        /// Input BPMN file
        input: String,

        /// Path to TINA's `sift` binary
        #[arg(long, env = "SIFT_PATH", default_value = "sift")]
        sift: String,

        /// Maximum number of markings to explore
        #[arg(long)]
        max_states: Option<usize>,

        /// Time limit for the analysis, in seconds
        #[arg(long)]
        timeout: Option<u64>,
    },

    /// Start HTTP server
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}
