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

    #[error("failed to write output file '{path}': {source}")]
    OutputFile {
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

fn write_output(path: &str, content: &str) -> Result<(), ConvertError> {
    fs::write(path, content).map_err(|e| ConvertError::OutputFile {
        path: path.to_string(),
        source: e,
    })
}

pub fn xml_to_net(xml: &str) -> Result<PetriNet, ConvertError> {
    let chor = parse(xml)?;
    encode(&chor).map_err(|errs| ConvertError::Encode(WellFormednessErrors(errs)))
}

pub fn convert_file(
    input: &str,
    output_pnml: Option<&str>,
    output_dot: Option<&str>,
) -> Result<PetriNet, ConvertError> {
    let xml = fs::read_to_string(input).map_err(|e| ConvertError::InputFile {
        path: input.to_string(),
        source: e,
    })?;

    let net = xml_to_net(&xml)?;

    if let Some(path) = output_pnml {
        write_output(path, &export_to_pnml(&net)?)?;
        println!("Exported PNML to {path}");
    }

    if let Some(path) = output_dot {
        write_output(path, &export_to_dot(&net)?)?;
        println!("Exported DOT to {path}");
    }

    if output_pnml.is_none() && output_dot.is_none() {
        println!("{net:?}");
    }

    Ok(net)
}
