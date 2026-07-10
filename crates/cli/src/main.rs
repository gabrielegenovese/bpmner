mod cli;
mod convert;
mod server;
use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Convert { input, pnml, dot } => {
            match convert::convert(&input, pnml.as_deref(), dot.as_deref()) {
                Ok(net) => println!("Conversion completed:\n\n{:?}", net),
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
