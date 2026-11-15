use crate::petri_net::pn::{Arc, PetriNet};
use std::collections::HashMap;
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

    #[error("failed to add initial marking while converting petri net: {0}")]
    InitialMarking(String),

    #[error("failed to convert to PNML: {0}")]
    ConvertPnml(String),

    #[error("failed to convert to DOT: {0}")]
    ConvertDot(String),
}

/// Options controlling how a Petri net is exported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportOptions {
    /// Include the net's initial marking in the output.
    pub initial_marking: bool,
    /// Lay out DOT graphs from left to right (`rankdir="LR"`).
    pub rankdir_lr: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            // The initial marking is what makes the exported net usable by
            // model checkers such as TINA/sift.
            initial_marking: true,
            rankdir_lr: false,
        }
    }
}

fn convert_to_pn_lib(
    mynet: &PetriNet,
    options: ExportOptions,
) -> Result<netcrab::petri_net::PetriNet, ExportError> {
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

    if options.initial_marking {
        for (place, tokens) in &mynet.marking {
            if *tokens == 0 {
                continue;
            }

            let place_ref = place_map
                .get(place)
                .ok_or_else(|| ExportError::MissingPlace(format!("{:?}", place)))?;

            new_net
                .add_token(place_ref, *tokens)
                .map_err(|e| ExportError::InitialMarking(e.to_string()))?;
        }
    }

    Ok(new_net)
}

fn add_rankdir(dot: String) -> String {
    dot.replacen(
        "digraph petrinet {\n",
        "digraph petrinet {\n    rankdir=\"LR\";\n",
        1,
    )
}

/// Exports a Petri net to PNML using the default options.
pub fn export_to_pnml(mynet: &PetriNet) -> Result<String, ExportError> {
    export_to_pnml_with_options(mynet, ExportOptions::default())
}

/// Exports a Petri net to PNML with explicit options.
pub fn export_to_pnml_with_options(
    mynet: &PetriNet,
    options: ExportOptions,
) -> Result<String, ExportError> {
    convert_to_pn_lib(mynet, options)?
        .to_pnml_string()
        .map_err(|e| ExportError::ConvertPnml(e.to_string()))
}

/// Exports a Petri net to Graphviz DOT using the default options.
pub fn export_to_dot(mynet: &PetriNet) -> Result<String, ExportError> {
    export_to_dot_with_options(mynet, ExportOptions::default())
}

/// Exports a Petri net to Graphviz DOT with explicit options.
pub fn export_to_dot_with_options(
    mynet: &PetriNet,
    options: ExportOptions,
) -> Result<String, ExportError> {
    let dot = convert_to_pn_lib(mynet, options)?
        .to_dot_string()
        .map_err(|e| ExportError::ConvertDot(e.to_string()))?;

    Ok(if options.rankdir_lr {
        add_rankdir(dot)
    } else {
        dot
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marked_net() -> PetriNet {
        PetriNet::new()
            .arc_pt("p_start_0", "t_start_0")
            .arc_tp("t_start_0", "e_0")
            .mark("p_start_0", 1)
    }

    #[test]
    fn pnml_includes_the_initial_marking_by_default() {
        let pnml = export_to_pnml(&marked_net()).expect("pnml export should succeed");
        assert!(pnml.contains("initialMarking"));
    }

    #[test]
    fn pnml_can_omit_the_initial_marking() {
        let pnml = export_to_pnml_with_options(
            &marked_net(),
            ExportOptions {
                initial_marking: false,
                rankdir_lr: false,
            },
        )
        .expect("pnml export should succeed");

        assert!(!pnml.contains("initialMarking"));
    }

    #[test]
    fn dot_can_use_a_left_to_right_layout() {
        let dot = export_to_dot_with_options(
            &marked_net(),
            ExportOptions {
                initial_marking: true,
                rankdir_lr: true,
            },
        )
        .expect("dot export should succeed");

        assert!(dot.contains("rankdir=\"LR\""));
    }
}
