mod cli;
mod convert;
mod server;
use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Convert { input, pnml, dot } => {
            match convert::convert_file(&input, pnml.as_deref(), dot.as_deref()) {
                Ok(_) => println!("Conversion completed successfully."),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }

        Command::Serve { port } => {
            if let Err(e) = server::run(port) {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
}
