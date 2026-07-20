use wf_core::bpmn::chor::parser::parse;
use wf_core::encoder::enc_chor::encode;
use wf_core::petri_net::exporter::{export_to_dot, export_to_pnml};

#[rustler::nif(schedule = "DirtyCpu")]
fn convert_bpmn_to_pnml(bpmn: String) -> Result<String, rustler::Error> {
    // BPMN -> Chor
    let chor = parse(&bpmn)
        .map_err(|e| rustler::Error::Term(Box::new(format!("parse error: {}", e.to_string()))))?;

    // Chor -> Petri Net
    let net = encode(&chor).map_err(|e| {
        rustler::Error::Term(Box::new(format!(
            "encode error: {}",
            e.iter()
                .map(|err| err.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )))
    })?;

    // Petri Net -> PNML
    export_to_pnml(&net)
        .map_err(|e| rustler::Error::Term(Box::new(format!("export error: {}", e.to_string()))))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn convert_bpmn_to_dot(bpmn: String) -> Result<String, rustler::Error> {
    // BPMN -> Chor
    let chor = parse(&bpmn)
        .map_err(|e| rustler::Error::Term(Box::new(format!("parse error: {}", e.to_string()))))?;

    // Chor -> Petri Net
    let net = encode(&chor).map_err(|e| {
        rustler::Error::Term(Box::new(format!(
            "encode error: {}",
            e.iter()
                .map(|err| err.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )))
    })?;

    // Petri Net -> DOT
    export_to_dot(&net)
        .map_err(|e| rustler::Error::Term(Box::new(format!("export error: {}", e.to_string()))))
}

rustler::init!("Elixir.Converter");
