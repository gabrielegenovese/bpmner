use crate::petri_net::pn::{Arc, PetriNet};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("failed to create output file '{path}': {source}")]
    FileCreation {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("missing place reference while exporting: {0}")]
    MissingPlace(String),

    #[error("missing transition reference while exporting: {0}")]
    MissingTransition(String),

    #[error("failed to add arc while converting petri net: {0}")]
    ArcConversion(String),

    #[error("failed to write PNML: {0}")]
    Pnml(String),

    #[error("failed to write DOT: {0}")]
    Dot(String),
}

fn convert_to_pn_lib(mynet: &PetriNet) -> Result<netcrab::petri_net::PetriNet, ExportError> {
    let mut new_net = netcrab::petri_net::PetriNet::new();

    let place_map = mynet.places.iter().fold(HashMap::new(), |mut map, p| {
        let place_ref = new_net.add_place(p);
        map.insert(p, place_ref);
        map
    });

    let transition_map = mynet.transitions.iter().fold(HashMap::new(), |mut map, t| {
        let transition_ref = new_net.add_transition(t);
        map.insert(t, transition_ref);
        map
    });

    mynet.flow.iter().try_for_each(|a| match a {
        Arc::PT(p, t) => {
            let place_ref = place_map
                .get(p)
                .ok_or_else(|| ExportError::MissingPlace(format!("{:?}", p)))?;

            let transition_ref = transition_map
                .get(t)
                .ok_or_else(|| ExportError::MissingTransition(format!("{:?}", t)))?;

            new_net
                .add_arc_place_transition(place_ref, transition_ref)
                .map_err(|e| ExportError::ArcConversion(e.to_string()))
        }

        Arc::TP(t, p) => {
            let transition_ref = transition_map
                .get(t)
                .ok_or_else(|| ExportError::MissingTransition(format!("{:?}", t)))?;

            let place_ref = place_map
                .get(p)
                .ok_or_else(|| ExportError::MissingPlace(format!("{:?}", p)))?;

            new_net
                .add_arc_transition_place(transition_ref, place_ref)
                .map_err(|e| ExportError::ArcConversion(e.to_string()))
        }
    })?;

    Ok(new_net)
}

pub fn export_to_pnml(path: &str, mynet: &PetriNet) -> Result<(), ExportError> {
    let mut buffer = File::create(path).map_err(|source| ExportError::FileCreation {
        path: path.to_string(),
        source,
    })?;

    convert_to_pn_lib(mynet)?
        .to_pnml(&mut buffer)
        .map_err(|e| ExportError::Pnml(e.to_string()))
}

pub fn export_to_dot(path: &str, mynet: &PetriNet) -> Result<(), ExportError> {
    let mut buffer = File::create(path).map_err(|source| ExportError::FileCreation {
        path: path.to_string(),
        source,
    })?;

    convert_to_pn_lib(mynet)?
        .to_dot(&mut buffer)
        .map_err(|e| ExportError::Dot(e.to_string()))
}
