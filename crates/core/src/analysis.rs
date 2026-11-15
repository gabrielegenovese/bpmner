//! External analysis of the encoded Petri net.
//!
//! The checks are delegated to [TINA](https://projects.laas.fr/tina/), in
//! particular to its `sift` state-space explorer. `sift` reads a PNML net from
//! standard input and stops as soon as it finds a state falsifying a
//! reachability formula.
//!
//! The formulas used here are:
//!
//! * safety: `L.safe`, an atomic proposition provided by `sift` asserting
//!   that no place ever carries more than one token;
//! * deadlock-freedom: `-L.dead \/ (final)`, where `final` is the conjunction
//!   of `-S.<place>` over every place. This accepts the (intended) final
//!   marking, in which all places are empty, and rejects any other deadlock.

use crate::petri_net::exporter::{ExportError, ExportOptions, export_to_pnml_with_options};
use crate::petri_net::pn::PetriNet;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Property {
    Safety,
    DeadlockFreeness,
}

impl Property {
    pub fn label(self) -> &'static str {
        match self {
            Property::Safety => "Safety (1-bounded)",
            Property::DeadlockFreeness => "Deadlock-freedom (final state allowed)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckOutcome {
    Holds,
    Violated,
    /// The analyzer stopped before exploring the whole state space.
    Unknown,
}

impl CheckOutcome {
    pub fn label(self) -> &'static str {
        match self {
            CheckOutcome::Holds => "holds",
            CheckOutcome::Violated => "violated",
            CheckOutcome::Unknown => "unknown",
        }
    }

    pub fn holds(self) -> bool {
        self == CheckOutcome::Holds
    }
}

#[derive(Debug, Clone)]
pub struct PropertyResult {
    pub property: Property,
    pub outcome: CheckOutcome,
    /// A short description of the offending state, when one was found.
    pub violation: Option<String>,
    /// Number of markings explored by `sift`, when reported.
    pub states: Option<usize>,
    /// Raw output of the analyzer, useful for debugging / logging.
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub safety: PropertyResult,
    pub deadlock: PropertyResult,
    pub places: usize,
    pub transitions: usize,
}

impl AnalysisReport {
    /// True when both properties were positively verified.
    pub fn all_hold(&self) -> bool {
        self.safety.outcome.holds() && self.deadlock.outcome.holds()
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    /// Path or name of the `sift` binary (TINA toolbox).
    pub sift_path: String,
    /// Limit the number of explored markings (`sift -c`).
    pub max_states: Option<usize>,
    /// Limit the exploration time in seconds (`sift -t`).
    pub time_limit: Option<u64>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            sift_path: "sift".to_string(),
            max_states: None,
            time_limit: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error(
        "could not run the Petri net analyzer '{path}': {source}. \
         Install TINA (https://projects.laas.fr/tina/) or point SIFT_PATH to the 'sift' binary"
    )]
    Spawn {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("the Petri net analyzer '{path}' rejected the net or the formula: {message}")]
    Tool { path: String, message: String },

    #[error("failed to communicate with the Petri net analyzer: {0}")]
    Io(#[from] io::Error),

    #[error("failed to export the net for analysis: {0}")]
    Export(#[from] ExportError),
}

/// Inputs needed to run the checks with an external tool.
#[derive(Debug, Clone)]
pub struct AnalysisInputs {
    pub pnml: String,
    pub safety_formula: String,
    pub deadlock_formula: String,
}

/// Builds the PNML document and the formulas used for the analysis.
pub fn inputs(net: &PetriNet) -> Result<AnalysisInputs, ExportError> {
    Ok(AnalysisInputs {
        pnml: export_to_pnml_with_options(
            net,
            ExportOptions {
                initial_marking: true,
                rankdir_lr: false,
            },
        )?,
        safety_formula: "L.safe".to_string(),
        deadlock_formula: final_marking_formula(net),
    })
}

fn final_marking_formula(net: &PetriNet) -> String {
    if net.places.is_empty() {
        return "T".to_string();
    }

    let mut places: Vec<&String> = net.places.iter().collect();
    places.sort();

    let empty_places = places
        .into_iter()
        .map(|place| format!("-S.{place}"))
        .collect::<Vec<_>>()
        .join(" /\\ ");

    format!("-L.dead \\/ ({empty_places})")
}

fn parse_states(output: &str) -> Option<usize> {
    output.lines().find_map(|line| {
        let idx = line.find(" marking(s)")?;
        line[..idx]
            .split_whitespace()
            .last()
            .and_then(|n| n.parse().ok())
    })
}

fn parse_violation(output: &str) -> Option<String> {
    let mut lines = output.lines();
    while let Some(line) = lines.next() {
        if line.contains("some state violates condition -f:") {
            let details = lines
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .take_while(|line| !line.contains("marking(s)"))
                .collect::<Vec<_>>()
                .join(" | ");

            return Some(if details.is_empty() {
                line.trim().to_string()
            } else {
                details
            });
        }
    }
    None
}

fn run_check(
    pnml: &str,
    formula: &str,
    property: Property,
    options: &AnalysisOptions,
) -> Result<PropertyResult, AnalysisError> {
    let mut command = Command::new(&options.sift_path);
    command.arg("-q").arg("-PNML").arg("-f").arg(formula);

    if let Some(max_states) = options.max_states {
        command.arg("-c").arg(max_states.to_string());
    }

    if let Some(time_limit) = options.time_limit {
        command.arg("-t").arg(time_limit.to_string());
    }

    // The single "-" makes sift read the net from standard input.
    command
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|source| AnalysisError::Spawn {
        path: options.sift_path.clone(),
        source,
    })?;

    {
        let stdin = child.stdin.as_mut().ok_or_else(|| AnalysisError::Tool {
            path: options.sift_path.clone(),
            message: "could not open the analyzer standard input".to_string(),
        })?;
        stdin.write_all(pnml.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let combined = format!("{stdout}{stderr}");

    let outcome = if combined.contains("some state violates condition -f:") {
        CheckOutcome::Violated
    } else if combined.contains("limit reached") {
        CheckOutcome::Unknown
    } else if combined.contains("marking(s)") {
        CheckOutcome::Holds
    } else {
        return Err(AnalysisError::Tool {
            path: options.sift_path.clone(),
            message: stderr.trim().to_string(),
        });
    };

    Ok(PropertyResult {
        property,
        outcome,
        violation: if outcome == CheckOutcome::Violated {
            parse_violation(&combined)
        } else {
            None
        },
        states: parse_states(&combined),
        output: {
            let mut log = format!("$ {} -q -PNML -f '{}' -\n", options.sift_path, formula);
            log.push_str(combined.trim_end());
            log
        },
    })
}

/// Runs the safety and deadlock-freedom checks with TINA's `sift`.
pub fn analyze(net: &PetriNet, options: &AnalysisOptions) -> Result<AnalysisReport, AnalysisError> {
    let analysis_inputs = inputs(net)?;

    let safety = run_check(
        &analysis_inputs.pnml,
        &analysis_inputs.safety_formula,
        Property::Safety,
        options,
    )?;
    let deadlock = run_check(
        &analysis_inputs.pnml,
        &analysis_inputs.deadlock_formula,
        Property::DeadlockFreeness,
        options,
    )?;

    Ok(AnalysisReport {
        safety,
        deadlock,
        places: net.places.len(),
        transitions: net.transitions.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_marking_formula_is_a_conjunction_of_empty_places() {
        let net = PetriNet::new().arc_pt("e_0", "t_0").mark("p_start_0", 1);
        let formula = final_marking_formula(&net);
        assert!(formula.contains("-S.e_0"));
        assert!(formula.contains("-S.p_start_0"));
        assert!(formula.contains("-L.dead"));
    }

    #[test]
    fn parses_state_counts() {
        let output = "565 marking(s), 1343 transition(s)\n0.001s\n";
        assert_eq!(parse_states(output), Some(565));
    }

    #[test]
    fn parses_violation_details() {
        let output = "some state violates condition -f:\n  e_2*2\n  firable: t_end_2\n9 marking(s), 9 transition(s)\n";
        assert_eq!(
            parse_violation(output).as_deref(),
            Some("e_2*2 | firable: t_end_2")
        );
    }
}
