use crate::bpmn::chor::{Choreography, ChoreographyEl};
use crate::bpmn::edge::ControlFlow;
use crate::petri_net::pn::{PetriNet, Place, Transition};
use std::collections::HashSet;

#[derive(Debug, Default, Clone, Copy)]
struct FreshIdGen {
    place_counter: usize,
    transition_counter: usize,
}

impl FreshIdGen {
    fn new() -> Self {
        Self::default()
    }

    fn fresh_transition(self, hint: &str) -> (Self, Transition) {
        let id = format!("t_{}_{}", hint, self.transition_counter);
        let next = Self {
            transition_counter: self.transition_counter + 1,
            ..self
        };
        (next, id)
    }

    fn fresh_place(self, hint: &str) -> (Self, Place) {
        let id = format!("p_{}_{}", hint, self.place_counter);
        let next = Self {
            place_counter: self.place_counter + 1,
            ..self
        };
        (next, id)
    }
}

fn negate(e: &ControlFlow) -> ControlFlow {
    ControlFlow::new(format!("not_{}", e.id()))
}

/// PN(start(e)) = (P, T, F)
/// P = {e, p_s} con p_s fresh
/// T = {t}
/// F = {(p_s, t), (t, e)}
fn encode_start(generator: FreshIdGen, e: &ControlFlow) -> (FreshIdGen, PetriNet) {
    let (generator, p_s) = generator.fresh_place("start");
    let (generator, t) = generator.fresh_transition("start");
    let net = PetriNet::new().arc_pt(p_s, t.clone()).arc_tp(t, e.id());
    (generator, net)
}

/// PN(end(e)) = (P, T, F)
/// P = {e}
/// T = {t}
/// F = {(e, t)}
fn encode_end(generator: FreshIdGen, e: &ControlFlow) -> (FreshIdGen, PetriNet) {
    let (generator, t) = generator.fresh_transition("end");
    let net = PetriNet::new().arc_pt(e.id(), t);
    (generator, net)
}

/// PN(task(e, e')) = (P, T, F)
/// P = {e, e'}
/// T = {t}
/// F = {(e, t), (t, e')}
fn encode_task(
    generator: FreshIdGen,
    e: &ControlFlow,
    e_prime: &ControlFlow,
) -> (FreshIdGen, PetriNet) {
    let (generator, t) = generator.fresh_transition("task");
    let net = PetriNet::new()
        .arc_pt(e.id(), t.clone())
        .arc_tp(t, e_prime.id());
    (generator, net)
}

/// PN(andSplit(e, E)) = (P, T, F)
/// P = {e} ∪ E
/// T = {t}
/// F = {(e, t)} ∪ {(t, e')}_{e' ∈ E}
fn encode_and_split(
    generator: FreshIdGen,
    e: &ControlFlow,
    outputs: &HashSet<ControlFlow>,
) -> (FreshIdGen, PetriNet) {
    let (generator, t) = generator.fresh_transition("and_split");
    let net = outputs
        .iter()
        .fold(PetriNet::new().arc_pt(e.id(), t.clone()), |net, e_prime| {
            net.arc_tp(t.clone(), e_prime.id())
        });
    (generator, net)
}

/// PN(andJoin(E, e)) = (P, T, F)
/// P = E ∪ {e}
/// T = {t}
/// F = {(e', t)}_{e' ∈ E} ∪ {(t, e)}
fn encode_and_join(
    generator: FreshIdGen,
    inputs: &HashSet<ControlFlow>,
    e: &ControlFlow,
) -> (FreshIdGen, PetriNet) {
    let (generator, t) = generator.fresh_transition("and_join");
    let net = inputs
        .iter()
        .fold(PetriNet::new(), |net, e_prime| {
            net.arc_pt(e_prime.id(), t.clone())
        })
        .arc_tp(t, e.id());
    (generator, net)
}

/// PN(xorSplit(e, E)) = (P, T, F)
/// P = {e} ∪ E ∪ {ē | e ∈ E}
/// T = {t_e | e ∈ E}
/// F = {(e, t_e), (t_e, e)}_{e∈E} ∪ {(t_e, ē') | e, e' ∈ E, e' ≠ e}
fn encode_xor_split(
    generator: FreshIdGen,
    e: &ControlFlow,
    outputs: &HashSet<ControlFlow>,
) -> (FreshIdGen, PetriNet) {
    let net = outputs.iter().fold(PetriNet::new(), |net, e_i| {
        let t_ei = format!("t_{}", e_i.id());

        let net = net
            .arc_pt(e.id(), t_ei.clone())
            .arc_tp(t_ei.clone(), e_i.id());

        outputs
            .iter()
            .filter(|e_j| *e_j != e_i)
            .fold(net, |net, e_j| net.arc_tp(t_ei.clone(), negate(e_j).id()))
    });
    (generator, net)
}

/// PN(xorJoin(E, e)) = (P, T, F)
/// P = E ∪ {e}
/// T = {t_e | e ∈ E}
/// F = {(e, t_e), (t_e, e)}_{e∈E}
fn encode_xor_join(
    generator: FreshIdGen,
    inputs: &HashSet<ControlFlow>,
    e: &ControlFlow,
) -> (FreshIdGen, PetriNet) {
    let net = inputs.iter().fold(PetriNet::new(), |net, e_i| {
        let t_ei = format!("t_{}", e_i.id());
        net.arc_pt(e_i.id(), t_ei.clone()).arc_tp(t_ei, e.id())
    });
    (generator, net)
}

fn encode_element(generator: FreshIdGen, element: &ChoreographyEl) -> (FreshIdGen, PetriNet) {
    match element {
        ChoreographyEl::Start { output } => encode_start(generator, output),
        ChoreographyEl::End { input } => encode_end(generator, input),
        ChoreographyEl::Task { input, output } => encode_task(generator, input, output),
        ChoreographyEl::AndSplit { input, output } => encode_and_split(generator, input, output),
        ChoreographyEl::AndJoin { inputs, output } => encode_and_join(generator, inputs, output),
        ChoreographyEl::XorSplit { input, output } => encode_xor_split(generator, input, output),
        ChoreographyEl::XorJoin { inputs, output } => encode_xor_join(generator, inputs, output),

        ChoreographyEl::OrSplit { .. } | ChoreographyEl::OrJoin { .. } => {
            todo!("encoding or not defined yet")
        }
    }
}

fn encode_choreography(
    generator: FreshIdGen,
    choreography: &Choreography,
) -> (FreshIdGen, PetriNet) {
    choreography
        .elements
        .iter()
        .fold((generator, PetriNet::new()), |(generator, net), el| {
            let (generator, el_net) = encode_element(generator, el);
            (generator, net.union(el_net))
        })
}

pub fn encode_with_init(choreography: &Choreography) -> PetriNet {
    let (_, net) = encode_choreography(FreshIdGen::new(), choreography);
    net
}
