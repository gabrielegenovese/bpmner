use crate::bpmn::edge::ControlFlow;
use crate::petri_net::pn::{Arc, PetriNet, Place, Transition};
use std::collections::{BTreeSet, HashSet};

#[derive(Debug, Default, Clone, Copy)]
pub struct FreshIdGen {
    place_counter: usize,
    transition_counter: usize,
}

impl FreshIdGen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fresh_transition(self, hint: &str) -> (Self, Transition) {
        let id = format!("t_{}_{}", hint, self.transition_counter);
        let next = Self {
            transition_counter: self.transition_counter + 1,
            ..self
        };
        (next, id)
    }

    pub fn fresh_place(self, hint: &str) -> (Self, Place) {
        let id = format!("p_{}_{}", hint, self.place_counter);
        let next = Self {
            place_counter: self.place_counter + 1,
            ..self
        };
        (next, id)
    }
}

pub fn powerset_non_empty(elements: &HashSet<ControlFlow>) -> Vec<HashSet<ControlFlow>> {
    elements
        .iter()
        .fold(vec![HashSet::new()], |subsets, e| {
            let with_e: Vec<HashSet<ControlFlow>> = subsets
                .iter()
                .map(|s| s.union(&HashSet::from([e.clone()])).cloned().collect())
                .collect();
            subsets.into_iter().chain(with_e).collect()
        })
        .into_iter()
        .filter(|s: &HashSet<ControlFlow>| !s.is_empty())
        .collect()
}

pub fn subset_transition_name(subset: &HashSet<ControlFlow>) -> Transition {
    let ids: BTreeSet<&str> = subset.iter().map(|e| e.id()).collect();
    format!("t_{}", ids.into_iter().collect::<Vec<_>>().join(""))
}

pub fn negate(e: &ControlFlow) -> ControlFlow {
    ControlFlow::new(format!("not_{}", e.id()))
}

fn not_place(p: &str) -> Place {
    format!("not_{}", p)
}

fn not_transition(t: &str) -> Transition {
    format!("not_{}", t)
}

pub fn encode_dead_propagation_net(pn: &PetriNet) -> PetriNet {
    pn.flow.iter().fold(PetriNet::new(), |net, arc| match arc {
        Arc::PT(p, t) => net.arc_pt(not_place(p), not_transition(t)),
        Arc::TP(t, p) => net.arc_tp(not_transition(t), not_place(p)),
    })
}
