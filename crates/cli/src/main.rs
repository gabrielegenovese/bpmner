mod cli;
mod convert;
mod server;
use clap::Parser;
use cli::{Cli, Command};
use wf_core::analysis::{AnalysisOptions, AnalysisReport, CheckOutcome, PropertyResult};
use wf_core::petri_net::exporter::ExportOptions;

fn print_property(result: &PropertyResult) {
    let status = match result.outcome {
        CheckOutcome::Holds => "holds",
        CheckOutcome::Violated => "VIOLATED",
        CheckOutcome::Unknown => "unknown (state space truncated)",
    };

    let states = result
        .states
        .map(|states| format!(" [{states} markings explored]"))
        .unwrap_or_default();

    println!("- {}: {status}{states}", result.property.label());

    if let Some(violation) = &result.violation {
        println!("  violating state: {violation}");
    }
}

fn print_report(report: &AnalysisReport) {
    println!(
        "Petri net: {} places, {} transitions",
        report.places, report.transitions
    );
    print_property(&report.safety);
    print_property(&report.deadlock);

    if !report.safety.outcome.holds() || !report.deadlock.outcome.holds() {
        println!();
        println!("Analyzer log:");
        for result in [&report.safety, &report.deadlock] {
            println!("{}", result.output);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Convert {
            input,
            pnml,
            dot,
            no_initial_marking,
            rankdir_lr,
        } => {
            let options = ExportOptions {
                initial_marking: !no_initial_marking,
                rankdir_lr,
            };

            match convert::convert_file(&input, pnml.as_deref(), dot.as_deref(), options) {
                Ok(_) => println!("Conversion completed successfully."),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }

        Command::Check {
            input,
            sift,
            max_states,
            timeout,
        } => {
            let options = AnalysisOptions {
                sift_path: sift,
                max_states,
                time_limit: timeout,
            };

            match convert::check_file(&input, options) {
                Ok(report) => {
                    print_report(&report);
                    if !report.all_hold() {
                        std::process::exit(1);
                    }
                }
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
