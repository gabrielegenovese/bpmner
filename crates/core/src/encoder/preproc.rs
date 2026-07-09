use crate::bpmn::chor::{Chor, ChorEl, edges_of};
use crate::bpmn::edge::ControlFlow;
use std::collections::HashSet;

fn entry_points(element: &ChorEl) -> HashSet<ControlFlow> {
    match element {
        ChorEl::Start { .. } => HashSet::new(),
        ChorEl::End { input } => HashSet::from([input.clone()]),
        ChorEl::Task { input, .. } => HashSet::from([input.clone()]),
        ChorEl::AndSplit { input, .. }
        | ChorEl::XorSplit { input, .. }
        | ChorEl::OrSplit { input, .. } => HashSet::from([input.clone()]),
        ChorEl::AndJoin { input, .. }
        | ChorEl::XorJoin { input, .. }
        | ChorEl::OrJoin { input, .. } => input.clone(),
    }
}

fn exit_points(element: &ChorEl) -> HashSet<ControlFlow> {
    match element {
        ChorEl::Start { output } => HashSet::from([output.clone()]),
        ChorEl::End { .. } => HashSet::new(),
        ChorEl::Task { output, .. } => HashSet::from([output.clone()]),
        ChorEl::AndSplit { output, .. }
        | ChorEl::XorSplit { output, .. }
        | ChorEl::OrSplit { output, .. } => output.clone(),
        ChorEl::AndJoin { output, .. }
        | ChorEl::XorJoin { output, .. }
        | ChorEl::OrJoin { output, .. } => HashSet::from([output.clone()]),
    }
}

fn is_split_seed(element: &ChorEl) -> bool {
    matches!(element, ChorEl::XorSplit { .. } | ChorEl::OrSplit { .. })
}

fn preproc_contribution(element: &ChorEl) -> HashSet<ControlFlow> {
    if is_split_seed(element) {
        exit_points(element)
    } else {
        edges_of(element)
    }
}

fn reachable_indices(
    exits: &[HashSet<ControlFlow>],
    entries: &[HashSet<ControlFlow>],
    frontier: HashSet<usize>,
) -> HashSet<usize> {
    let next: HashSet<usize> = frontier
        .iter()
        .flat_map(|&i| {
            entries
                .iter()
                .enumerate()
                .filter(move |(_, entry_j)| !exits[i].is_disjoint(entry_j))
                .map(|(j, _)| j)
        })
        .chain(frontier.iter().copied())
        .collect();

    if next == frontier {
        frontier
    } else {
        reachable_indices(exits, entries, next)
    }
}

pub fn preproc(chor: &Chor) -> HashSet<ControlFlow> {
    let elements = &chor.elements;

    let entries: Vec<HashSet<ControlFlow>> = elements.iter().map(entry_points).collect();
    let exits: Vec<HashSet<ControlFlow>> = elements.iter().map(exit_points).collect();

    let seeds: HashSet<usize> = elements
        .iter()
        .enumerate()
        .filter_map(|(i, el)| is_split_seed(el).then_some(i))
        .collect();

    let reachable = reachable_indices(&exits, &entries, seeds);

    reachable
        .iter()
        .flat_map(|&i| preproc_contribution(&elements[i]))
        .collect()
}
