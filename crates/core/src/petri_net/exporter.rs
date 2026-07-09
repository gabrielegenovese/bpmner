use crate::petri_net::pn::{Arc, PetriNet};
use std::collections::HashMap;
use std::fs::File;

fn convert_to_pn_lib(mynet: &PetriNet) -> netcrab::petri_net::PetriNet {
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

    mynet.flow.iter().for_each(|a| match a {
        Arc::PT(p, t) => {
            new_net
                .add_arc_place_transition(place_map.get(p).unwrap(), transition_map.get(t).unwrap())
                .unwrap();
        }
        Arc::TP(t, p) => {
            new_net
                .add_arc_transition_place(transition_map.get(t).unwrap(), place_map.get(p).unwrap())
                .unwrap();
        }
    });

    new_net
}

pub fn export_to_pnml(path: &str, mynet: &PetriNet) {
    let mut buffer = File::create(path).unwrap();
    convert_to_pn_lib(mynet).to_pnml(&mut buffer).unwrap();
}

pub fn export_to_dot(path: &str, mynet: &PetriNet) {
    let mut buffer = File::create(path).unwrap();
    convert_to_pn_lib(mynet).to_dot(&mut buffer).unwrap();
}
