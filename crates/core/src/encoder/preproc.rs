use crate::bpmn::chor::syntax::{Chor, ChorEl, edges_of};
use crate::bpmn::edge::ControlFlow;
use crate::encoder::util::FreshIdGen;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/* Find the set of edges after a XOR- or OR-split */

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

/* Well-formedness conditions */

#[derive(Debug, Clone, Error)]
pub enum WellFormednessError {
    #[error("choreography has no start element")]
    NoStart,

    #[error("choreography has multiple start elements ({0})")]
    MultipleStart(usize),

    #[error("choreography has no end element")]
    NoEnd,

    #[error("control flow edge is not a unique entry edge: {edge:?} appears {count} times")]
    EdgeNotUniqueEntry { edge: ControlFlow, count: usize },

    #[error("control flow edge is not a unique exit edge: {edge:?} appears {count} times")]
    EdgeNotUniqueExit { edge: ControlFlow, count: usize },

    #[error("unreachable terminal element at index {index}: {element:?}")]
    UnreachableTerm { index: usize, element: ChorEl },
}

fn count_occurrences<'a>(
    sets: impl Iterator<Item = &'a HashSet<ControlFlow>>,
) -> HashMap<ControlFlow, usize> {
    sets.flatten().fold(HashMap::new(), |mut acc, e| {
        *acc.entry(e.clone()).or_insert(0) += 1;
        acc
    })
}

// every edge appears exactly once as entry point and exactly once as exit point
fn check_edge_uniqueness(elements: &[ChorEl]) -> Vec<WellFormednessError> {
    let entry_counts =
        count_occurrences(elements.iter().map(entry_points).collect::<Vec<_>>().iter());
    let exit_counts =
        count_occurrences(elements.iter().map(exit_points).collect::<Vec<_>>().iter());

    let all_edges: HashSet<ControlFlow> = entry_counts
        .keys()
        .chain(exit_counts.keys())
        .cloned()
        .collect();

    all_edges
        .into_iter()
        .flat_map(|e| {
            let entry_err = match entry_counts.get(&e).copied().unwrap_or(0) {
                1 => None,
                n => Some(WellFormednessError::EdgeNotUniqueEntry {
                    edge: e.clone(),
                    count: n,
                }),
            };
            let exit_err = match exit_counts.get(&e).copied().unwrap_or(0) {
                1 => None,
                n => Some(WellFormednessError::EdgeNotUniqueExit { edge: e, count: n }),
            };
            entry_err.into_iter().chain(exit_err)
        })
        .collect()
}

// unique occurrence of start(e) and at least one occurrence of end(e)
fn check_start_end(elements: &[ChorEl]) -> Vec<WellFormednessError> {
    let start_count = elements
        .iter()
        .filter(|e| matches!(e, ChorEl::Start { .. }))
        .count();
    let end_count = elements
        .iter()
        .filter(|e| matches!(e, ChorEl::End { .. }))
        .count();

    let start_err = match start_count {
        1 => None,
        0 => Some(WellFormednessError::NoStart),
        n => Some(WellFormednessError::MultipleStart(n)),
    };

    let end_err = (end_count == 0).then_some(WellFormednessError::NoEnd);

    start_err.into_iter().chain(end_err).collect()
}

// every basic term is syntactically reachable
fn check_reachability(elements: &[ChorEl]) -> Vec<WellFormednessError> {
    let entries: Vec<HashSet<ControlFlow>> = elements.iter().map(entry_points).collect();
    let exits: Vec<HashSet<ControlFlow>> = elements.iter().map(exit_points).collect();

    let seeds: HashSet<usize> = elements
        .iter()
        .enumerate()
        .filter_map(|(i, el)| matches!(el, ChorEl::Start { .. }).then_some(i))
        .collect();

    let reachable = reachable_indices(&exits, &entries, seeds);

    (0..elements.len())
        .filter(|i| !reachable.contains(i))
        .map(|i| WellFormednessError::UnreachableTerm {
            index: i,
            element: elements[i].clone(),
        })
        .collect()
}

pub fn check_well_formed(chor: &Chor) -> Result<(), Vec<WellFormednessError>> {
    let elements = &chor.elements;

    // println!("Checking chor {:?}", chor);

    let errors: Vec<WellFormednessError> = check_start_end(elements)
        .into_iter()
        .chain(check_edge_uniqueness(elements))
        .chain(check_reachability(elements))
        .collect();

    // println!("Checking error {:?} ", errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/* Canonicalization */

fn is_start(element: &ChorEl) -> bool {
    matches!(element, ChorEl::Start { .. })
}

fn visit_edge(
    edge: &ControlFlow,
    elements: &[ChorEl],
    entries: &[HashSet<ControlFlow>],
    exits: &[HashSet<ControlFlow>],
    generator: FreshIdGen,
    mapping: HashMap<ControlFlow, ControlFlow>,
) -> (FreshIdGen, HashMap<ControlFlow, ControlFlow>) {
    if mapping.contains_key(edge) {
        return (generator, mapping);
    }

    let (generator, fresh) = generator.fresh_edge();
    let mapping = mapping
        .into_iter()
        .chain(std::iter::once((edge.clone(), fresh)))
        .collect();

    let successors: Vec<ControlFlow> = elements
        .iter()
        .enumerate()
        .filter(|(i, _)| entries[*i].contains(edge))
        .flat_map(|(i, _)| exits[i].iter().cloned())
        .collect();

    successors
        .iter()
        .fold((generator, mapping), |(generator, mapping), next_edge| {
            visit_edge(next_edge, elements, entries, exits, generator, mapping)
        })
}

fn build_edge_mapping(chor: &Chor) -> HashMap<ControlFlow, ControlFlow> {
    let elements = &chor.elements;
    let entries: Vec<HashSet<ControlFlow>> = elements.iter().map(entry_points).collect();
    let exits: Vec<HashSet<ControlFlow>> = elements.iter().map(exit_points).collect();

    let start_edges: Vec<ControlFlow> = elements
        .iter()
        .filter(|element| is_start(element))
        .flat_map(exit_points)
        .collect();

    start_edges
        .iter()
        .fold(
            (FreshIdGen::new(), HashMap::new()),
            |(generator, mapping), edge| {
                visit_edge(edge, elements, &entries, &exits, generator, mapping)
            },
        )
        .1
}

fn rename(edge: &ControlFlow, mapping: &HashMap<ControlFlow, ControlFlow>) -> ControlFlow {
    mapping.get(edge).cloned().unwrap_or_else(|| edge.clone())
}

fn rename_set(
    edges: &HashSet<ControlFlow>,
    mapping: &HashMap<ControlFlow, ControlFlow>,
) -> HashSet<ControlFlow> {
    edges.iter().map(|e| rename(e, mapping)).collect()
}

fn apply_renaming(element: &ChorEl, mapping: &HashMap<ControlFlow, ControlFlow>) -> ChorEl {
    match element {
        ChorEl::Start { output } => ChorEl::Start {
            output: rename(output, mapping),
        },

        ChorEl::End { input } => ChorEl::End {
            input: rename(input, mapping),
        },

        ChorEl::Task { input, output, .. } => ChorEl::Task {
            input: rename(input, mapping),
            output: rename(output, mapping),
        },

        ChorEl::AndSplit { input, output, .. } => ChorEl::AndSplit {
            input: rename(input, mapping),
            output: rename_set(output, mapping),
        },

        ChorEl::XorSplit { input, output, .. } => ChorEl::XorSplit {
            input: rename(input, mapping),
            output: rename_set(output, mapping),
        },

        ChorEl::OrSplit { input, output, .. } => ChorEl::OrSplit {
            input: rename(input, mapping),
            output: rename_set(output, mapping),
        },

        ChorEl::AndJoin { input, output, .. } => ChorEl::AndJoin {
            input: rename_set(input, mapping),
            output: rename(output, mapping),
        },

        ChorEl::XorJoin { input, output, .. } => ChorEl::XorJoin {
            input: rename_set(input, mapping),
            output: rename(output, mapping),
        },

        ChorEl::OrJoin { input, output, .. } => ChorEl::OrJoin {
            input: rename_set(input, mapping),
            output: rename(output, mapping),
        },
    }
}

/// Renames every edge in the choreography to a canonical `e_i` label.
pub fn canonicalize_edges(chor: &Chor) -> Chor {
    let mapping = build_edge_mapping(chor);

    let elements = chor
        .elements
        .iter()
        .map(|element| apply_renaming(element, &mapping))
        .collect();

    Chor { elements }
}
