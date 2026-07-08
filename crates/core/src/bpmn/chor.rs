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
pub enum ChoreographyEl {
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
pub struct Choreography {
    pub elements: Vec<ChoreographyEl>,
}
