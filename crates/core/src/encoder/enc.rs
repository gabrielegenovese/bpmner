use crate::bpmn::chor::{Chor, ChorEl, edges_of};
use crate::bpmn::edge::ControlFlow;
use crate::encoder::preproc::preproc;
use crate::encoder::util::{
    FreshIdGen, encode_dead_propagation_net, negate, powerset_non_empty, subset_transition_name,
};
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

/// PN(orSplit(e, E)) = (P, T, F)
/// P = {e} ∪ E ∪ {ē' | e' ∈ E}
/// T = {t_S | S ∈ P_∅(E)}
/// F = {(e, t_S)}_{S} ∪ {(t_S, e')}_{S, e'∈S} ∪ {(t_S, ē')}_{S, e'∈E\S}
fn encode_or_split(
    generator: FreshIdGen,
    input: &ControlFlow,
    output: &HashSet<ControlFlow>,
) -> (FreshIdGen, PetriNet) {
    let net = powerset_non_empty(output)
        .iter()
        .fold(PetriNet::new(), |net, subset| {
            let t_s = subset_transition_name(subset);

            let net = net.arc_pt(input.id(), t_s.clone());

            let net = subset
                .iter()
                .fold(net, |net, e| net.arc_tp(t_s.clone(), e.id()));

            output
                .difference(subset)
                .fold(net, |net, e| net.arc_tp(t_s.clone(), negate(e).id()))
        });

    (generator, net)
}

/// PN(orJoin(E, e)) = (P, T, F)
/// P = {e} ∪ E ∪ {ē' | e' ∈ E}
/// T = {t_S | S ∈ P_∅(E)}
/// F = {(t_S, e)}_{S} ∪ {(e', t_S)}_{S, e'∈S} ∪ {(ē', t_S)}_{S, e'∈E\S}
fn encode_or_join(
    generator: FreshIdGen,
    input: &HashSet<ControlFlow>,
    output: &ControlFlow,
) -> (FreshIdGen, PetriNet) {
    let net = powerset_non_empty(input)
        .iter()
        .fold(PetriNet::new(), |net, subset| {
            let t_s = subset_transition_name(subset);

            let net = subset
                .iter()
                .fold(net, |net, e| net.arc_pt(e.id(), t_s.clone()));

            let net = input
                .difference(subset)
                .fold(net, |net, e| net.arc_pt(negate(e).id(), t_s.clone()));

            net.arc_tp(t_s, output.id())
        });

    (generator, net)
}

fn encode_element(generator: FreshIdGen, element: &ChorEl) -> (FreshIdGen, PetriNet) {
    match element {
        ChorEl::Start { output } => encode_start(generator, output),
        ChorEl::End { input } => encode_end(generator, input),
        ChorEl::Task { input, output } => encode_task(generator, input, output),
        ChorEl::AndSplit { input, output } => encode_and_split(generator, input, output),
        ChorEl::AndJoin { input, output } => encode_and_join(generator, input, output),
        ChorEl::XorSplit { input, output } => encode_xor_split(generator, input, output),
        ChorEl::XorJoin { input, output } => encode_xor_join(generator, input, output),
        ChorEl::OrSplit { input, output } => encode_or_split(generator, input, output),
        ChorEl::OrJoin { input, output } => encode_or_join(generator, input, output),
    }
}

fn encode_dead_propagation(
    generator: FreshIdGen,
    element: &ChorEl,
    pn: &PetriNet,
) -> (FreshIdGen, PetriNet) {
    match element {
        ChorEl::XorSplit { input, output } | ChorEl::OrSplit { input, output } => {
            let (generator, t_bar) = generator.fresh_transition("not_split");

            let net = output.iter().fold(
                PetriNet::new().arc_pt(negate(input).id(), t_bar.clone()),
                |net, e| net.arc_tp(t_bar.clone(), negate(e).id()),
            );

            (generator, net)
        }

        ChorEl::XorJoin { input, output } | ChorEl::OrJoin { input, output } => {
            let (generator, t_bar) = generator.fresh_transition("not_join");

            let net = input
                .iter()
                .fold(PetriNet::new(), |net, e| {
                    net.arc_pt(negate(e).id(), t_bar.clone())
                })
                .arc_tp(t_bar, negate(output).id());

            (generator, net)
        }

        _ => (generator, encode_dead_propagation_net(pn)),
    }
}

fn encode_component(
    generator: FreshIdGen,
    element: &ChorEl,
    preproc: &HashSet<ControlFlow>,
) -> (FreshIdGen, PetriNet) {
    let (generator, pn) = encode_element(generator, element);
    let needs_dead_propagation = edges_of(element).is_subset(preproc);

    if needs_dead_propagation {
        let (generator, dead_pn) = encode_dead_propagation(generator, element, &pn);
        (generator, pn.union(dead_pn))
    } else {
        (generator, pn)
    }
}

fn encode_chor_full(
    generator: FreshIdGen,
    chor: &Chor,
    preproc: HashSet<ControlFlow>,
) -> (FreshIdGen, PetriNet) {
    chor.elements
        .iter()
        .fold((generator, PetriNet::new()), |(generator, net), el| {
            let (generator, el_net) = encode_component(generator, el, &preproc);
            (generator, net.union(el_net))
        })
}

pub fn encode_with_init(chor: &Chor) -> PetriNet {
    encode_chor_full(FreshIdGen::new(), chor, preproc(chor)).1
}
