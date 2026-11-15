use serde_json::json;
use wf_core::analysis::inputs;
use wf_core::bpmn::chor::parser::parse;
use wf_core::encoder::enc_chor::encode;
use wf_core::petri_net::exporter::{
    ExportOptions, export_to_dot_with_options, export_to_pnml_with_options,
};
use wf_core::petri_net::pn::PetriNet;

fn to_net(bpmn: &str) -> Result<PetriNet, String> {
    // BPMN -> Chor
    let chor = parse(bpmn).map_err(|e| format!("parse error: {e}"))?;

    // Chor -> Petri Net
    encode(&chor).map_err(|errors| {
        format!(
            "encode error: {}",
            errors
                .iter()
                .map(|err| err.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

#[rustler::nif(schedule = "DirtyCpu")]
fn convert_bpmn_to_pnml(bpmn: String, initial_marking: bool) -> Result<String, String> {
    let net = to_net(&bpmn)?;

    export_to_pnml_with_options(
        &net,
        ExportOptions {
            initial_marking,
            rankdir_lr: false,
        },
    )
    .map_err(|e| format!("export error: {e}"))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn convert_bpmn_to_dot(
    bpmn: String,
    initial_marking: bool,
    rankdir_lr: bool,
) -> Result<String, String> {
    let net = to_net(&bpmn)?;

    export_to_dot_with_options(
        &net,
        ExportOptions {
            initial_marking,
            rankdir_lr,
        },
    )
    .map_err(|e| format!("export error: {e}"))
}

/// Returns everything the Elixir side needs to run TINA's `sift`.
///
/// Running an external process from a NIF is not reliable on the BEAM: the
/// runtime reaps child processes, so `wait` fails with
/// `No child processes (os error 10)`. The Elixir side therefore owns the
/// process and only asks the NIF for the PNML document and the formulas.
#[rustler::nif(schedule = "DirtyCpu")]
fn analysis_inputs(bpmn: String) -> Result<String, String> {
    let net = to_net(&bpmn)?;
    let inputs = inputs(&net).map_err(|e| format!("export error: {e}"))?;

    Ok(json!({
        "pnml": inputs.pnml,
        "safety_formula": inputs.safety_formula,
        "deadlock_formula": inputs.deadlock_formula,
        "places": net.places.len(),
        "transitions": net.transitions.len(),
    })
    .to_string())
}

rustler::init!("Elixir.Converter");
