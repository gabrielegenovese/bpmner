use crate::bpmn::edge::ControlFlow;
use std::collections::HashSet;

/// BPMN Collaboration elements.
///
/// Corresponds exactly to the grammar:
///
/// start(e)
/// end(e)
/// task(e,e')  TODO: distinguish from Collabeography
/// andSplit(e,E)
/// andJoin(E,e)
/// xorSplit(e,E)
/// xorJoin(E,e)
/// TODO: ADD UNIQUE COLLABORATION ELEMENTS (event based gateway, send event, receive event)
#[derive(Debug, Clone)]
pub enum CollabEl {
    Start {
        output: ControlFlow,
    },
    End {
        input: ControlFlow,
    },
    Task {
        input: ControlFlow,
        output: ControlFlow,
    },
    AndSplit {
        input: ControlFlow,
        output: HashSet<ControlFlow>,
    },
    AndJoin {
        input: HashSet<ControlFlow>,
        output: ControlFlow,
    },
    XorSplit {
        input: ControlFlow,
        output: HashSet<ControlFlow>,
    },
    XorJoin {
        input: HashSet<ControlFlow>,
        output: ControlFlow,
    },
}

/// A BPMN Collaboration is the parallel composition of basic terms.
#[derive(Debug, Clone, Default)]
pub struct Collab {
    pub elements: Vec<CollabEl>,
}

impl Collab {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn edges_of(element: &CollabEl) -> HashSet<ControlFlow> {
    match element {
        CollabEl::Start { output } => HashSet::from([output.clone()]),
        CollabEl::End { input } => HashSet::from([input.clone()]),
        CollabEl::Task { input, output } => HashSet::from([input.clone(), output.clone()]),
        CollabEl::AndSplit { input, output } | CollabEl::XorSplit { input, output } => output
            .iter()
            .cloned()
            .chain(std::iter::once(input.clone()))
            .collect(),
        CollabEl::AndJoin { input, output } | CollabEl::XorJoin { input, output } => input
            .iter()
            .cloned()
            .chain(std::iter::once(output.clone()))
            .collect(),
    }
}
