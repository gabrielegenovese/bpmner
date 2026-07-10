use crate::bpmn::edge::ControlFlow;
use std::collections::HashSet;

/// BPMN Choreography elements.
///
/// Corresponds exactly to the grammar:
///
/// start(e)
/// end(e)
/// task(e,e')
/// andSplit(e,E)
/// andJoin(E,e)
/// xorSplit(e,E)
/// xorJoin(E,e)
/// orSplit(e,E)
/// orJoin(E,e)
#[derive(Debug, Clone)]
pub enum ChorEl {
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
    OrSplit {
        input: ControlFlow,
        output: HashSet<ControlFlow>,
    },
    OrJoin {
        input: HashSet<ControlFlow>,
        output: ControlFlow,
    },
}

/// A BPMN choreography is the parallel composition of basic terms.
#[derive(Debug, Clone, Default)]
pub struct Chor {
    pub elements: Vec<ChorEl>,
}

impl Chor {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn edges_of(element: &ChorEl) -> HashSet<ControlFlow> {
    match element {
        ChorEl::Start { output } => HashSet::from([output.clone()]),
        ChorEl::End { input } => HashSet::from([input.clone()]),
        ChorEl::Task { input, output } => HashSet::from([input.clone(), output.clone()]),
        ChorEl::AndSplit { input, output }
        | ChorEl::XorSplit { input, output }
        | ChorEl::OrSplit { input, output } => output
            .iter()
            .cloned()
            .chain(std::iter::once(input.clone()))
            .collect(),
        ChorEl::AndJoin { input, output }
        | ChorEl::XorJoin { input, output }
        | ChorEl::OrJoin { input, output } => input
            .iter()
            .cloned()
            .chain(std::iter::once(output.clone()))
            .collect(),
    }
}
