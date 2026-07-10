use crate::bpmn::collab::syntax::{Collab, CollabEl};
use crate::bpmn::edge::ControlFlow;
use crate::encoder::util::{FreshIdGen, negate};
use crate::petri_net::pn::PetriNet;
use std::collections::HashSet;

/// PN(start(e)) = (P, T, F)
/// P = {e, p_s} con p_s fresh
/// T = {t}
/// F = {(p_s, t), (t, e)}
fn encode_start(generator: FreshIdGen, output: &ControlFlow) -> (FreshIdGen, PetriNet) {
    let (generator, p_s) = generator.fresh_place("start");
    let (generator, t) = generator.fresh_transition("start");
    let net = PetriNet::new()
        .arc_pt(p_s, t.clone())
        .arc_tp(t, output.id());
    (generator, net)
}

/// PN(end(e)) = (P, T, F)
/// P = {e}
/// T = {t}
/// F = {(e, t)}
fn encode_end(generator: FreshIdGen, input: &ControlFlow) -> (FreshIdGen, PetriNet) {
    let (generator, t) = generator.fresh_transition("end");
    let net = PetriNet::new().arc_pt(input.id(), t);
    (generator, net)
}

/// PN(task(e, e')) = (P, T, F)
/// P = {e, e'}
/// T = {t}
/// F = {(e, t), (t, e')}
fn encode_task(
    generator: FreshIdGen,
    input: &ControlFlow,
    output: &ControlFlow,
) -> (FreshIdGen, PetriNet) {
    let (generator, t) = generator.fresh_transition("task");
    let net = PetriNet::new()
        .arc_pt(input.id(), t.clone())
        .arc_tp(t, output.id());
    (generator, net)
}

/// PN(andSplit(e, E)) = (P, T, F)
/// P = {e} ∪ E
/// T = {t}
/// F = {(e, t)} ∪ {(t, e')}_{e' ∈ E}
fn encode_and_split(
    generator: FreshIdGen,
    input: &ControlFlow,
    output: &HashSet<ControlFlow>,
) -> (FreshIdGen, PetriNet) {
    let (generator, t) = generator.fresh_transition("and_split");
    let net = output
        .iter()
        .fold(PetriNet::new().arc_pt(input.id(), t.clone()), |net, e| {
            net.arc_tp(t.clone(), e.id())
        });
    (generator, net)
}

/// PN(andJoin(E, e)) = (P, T, F)
/// P = E ∪ {e}
/// T = {t}
/// F = {(e', t)}_{e' ∈ E} ∪ {(t, e)}
fn encode_and_join(
    generator: FreshIdGen,
    input: &HashSet<ControlFlow>,
    output: &ControlFlow,
) -> (FreshIdGen, PetriNet) {
    let (generator, t) = generator.fresh_transition("and_join");
    let net = input
        .iter()
        .fold(PetriNet::new(), |net, e| net.arc_pt(e.id(), t.clone()))
        .arc_tp(t, output.id());
    (generator, net)
}

/// PN(xorSplit(e, E)) = (P, T, F)
/// P = {e} ∪ E ∪ {ē | e ∈ E}
/// T = {t_e | e ∈ E}
/// F = {(e, t_e), (t_e, e)}_{e∈E} ∪ {(t_e, ē') | e, e' ∈ E, e' ≠ e}
fn encode_xor_split(
    generator: FreshIdGen,
    input: &ControlFlow,
    output: &HashSet<ControlFlow>,
) -> (FreshIdGen, PetriNet) {
    let net = output.iter().fold(PetriNet::new(), |net, e| {
        let t_ei = format!("t_{}", e.id());

        let net = net
            .arc_pt(input.id(), t_ei.clone())
            .arc_tp(t_ei.clone(), e.id());

        output
            .iter()
            .filter(|e1| *e1 != e)
            .fold(net, |net, e1| net.arc_tp(t_ei.clone(), negate(e1).id()))
    });
    (generator, net)
}

/// PN(xorJoin(E, e)) = (P, T, F)
/// P = E ∪ {e}
/// T = {t_e | e ∈ E}
/// F = {(e, t_e), (t_e, e)}_{e∈E}
fn encode_xor_join(
    generator: FreshIdGen,
    input: &HashSet<ControlFlow>,
    output: &ControlFlow,
) -> (FreshIdGen, PetriNet) {
    let net = input.iter().fold(PetriNet::new(), |net, e| {
        let t_ei = format!("t_{}", e.id());
        net.arc_pt(e.id(), t_ei.clone()).arc_tp(t_ei, output.id())
    });
    (generator, net)
}

fn encode_element(generator: FreshIdGen, element: &CollabEl) -> (FreshIdGen, PetriNet) {
    match element {
        CollabEl::Start { output } => encode_start(generator, output),
        CollabEl::End { input } => encode_end(generator, input),
        CollabEl::Task { input, output } => encode_task(generator, input, output),
        CollabEl::AndSplit { input, output } => encode_and_split(generator, input, output),
        CollabEl::AndJoin { input, output } => encode_and_join(generator, input, output),
        CollabEl::XorSplit { input, output } => encode_xor_split(generator, input, output),
        CollabEl::XorJoin { input, output } => encode_xor_join(generator, input, output),
    }
}

fn encode_full(generator: FreshIdGen, collab: &Collab) -> (FreshIdGen, PetriNet) {
    collab
        .elements
        .iter()
        .fold((generator, PetriNet::new()), |(generator, net), el| {
            let (generator, el_net) = encode_element(generator, el);
            (generator, net.union(el_net))
        })
}

pub fn encode(collab: &Collab) -> PetriNet {
    encode_full(FreshIdGen::new(), collab).1
}
