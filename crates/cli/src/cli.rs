use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Convert a BPMN file
    Convert {
        /// Input BPMN file
        input: String,

        /// Output PNML
        #[arg(long)]
        pnml: Option<String>,

        /// Output DOT
        #[arg(long)]
        dot: Option<String>,
    },

    /// Start HTTP server
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}
