use crate::petri_net::pn::{Arc, PetriNet};
use std::collections::HashMap;
use std::fs::File;

fn convert_to_pn_lib(mynet: &PetriNet) -> Result<netcrab::petri_net::PetriNet, String> {
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
                .ok_or_else(|| "missing place reference in map".to_string())?;
            let transition_ref = transition_map
                .get(t)
                .ok_or_else(|| "missing transition reference in map".to_string())?;

            new_net
                .add_arc_place_transition(place_ref, transition_ref)
                .map_err(|e| e.to_string())
        }
        Arc::TP(p, t) => {
            let transition_ref = transition_map
                .get(t)
                .ok_or_else(|| "missing transition reference in map".to_string())?;
            let place_ref = place_map
                .get(p)
                .ok_or_else(|| "missing place reference in map".to_string())?;

            new_net
                .add_arc_transition_place(transition_ref, place_ref)
                .map_err(|e| e.to_string())
        }
    })?;

    Ok(new_net)
}

pub fn export_to_pnml(path: &str, mynet: &PetriNet) -> Result<(), String> {
    let mut buffer = File::create(path).map_err(|e| e.to_string())?;
    convert_to_pn_lib(mynet)?
        .to_pnml(&mut buffer)
        .map_err(|e| e.to_string())
}

pub fn export_to_dot(path: &str, mynet: &PetriNet) -> Result<(), String> {
    let mut buffer = File::create(path).map_err(|e| e.to_string())?;
    convert_to_pn_lib(mynet)?
        .to_dot(&mut buffer)
        .map_err(|e| e.to_string())
}
