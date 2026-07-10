use std::fs;
use thiserror::Error;
use wf_core::bpmn::chor::parser::{ParseError, parse};
use wf_core::encoder::enc_chor::encode;
use wf_core::encoder::preproc::WellFormednessError;
use wf_core::petri_net::exporter::{ExportError, export_to_dot, export_to_pnml};
use wf_core::petri_net::pn::PetriNet;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("BPMN parsing failed: {0}")]
    Parse(#[from] ParseError),

    #[error("Petri net encoding failed for structural conditions: {0}")]
    Encode(#[from] WellFormednessErrors),

    #[error("Export failed: {0}")]
    Export(#[from] ExportError),

    #[error("failed to read input file '{path}': {source}")]
    InputFile {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug)]
pub struct WellFormednessErrors(pub Vec<WellFormednessError>);

impl std::fmt::Display for WellFormednessErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} error(s):", self.0.len())?;
        for err in &self.0 {
            writeln!(f, "  - {}", err)?;
        }
        Ok(())
    }
}

impl std::error::Error for WellFormednessErrors {}

pub fn convert(
    input: &str,
    output_pnml: Option<&str>,
    output_dot: Option<&str>,
) -> Result<PetriNet, ConvertError> {
    let xml = fs::read_to_string(input).map_err(|e| ConvertError::InputFile {
        path: input.to_string(),
        source: e,
    })?;

    let chor = parse(&xml)?;
    let net: PetriNet = encode(&chor).map_err(WellFormednessErrors)?;

    if let Some(path) = output_pnml {
        export_to_pnml(path, &net).map_err(ConvertError::Export)?;
    }

    if let Some(path) = output_dot {
        export_to_dot(path, &net).map_err(ConvertError::Export)?;
    }

    Ok(net)
}
