use crate::bpmn::common::{CFlow};
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
pub enum ChoreographyElement {
    Start {
        out: CFlow,
    },
    End {
        input: CFlow,
    },
    Task {
        input: CFlow,
        output: CFlow,
    },
    AndSplit {
        input: CFlow,
        outputs: HashSet<CFlow>,
    },
    AndJoin {
        inputs: HashSet<CFlow>,
        output: CFlow,
    },
    XorSplit {
        input: CFlow,
        outputs: HashSet<CFlow>,
    },
    XorJoin {
        inputs: HashSet<CFlow>,
        output: CFlow,
    },
    OrSplit {
        input: CFlow,
        outputs: HashSet<CFlow>,
    },
    OrJoin {
        inputs: HashSet<CFlow>,
        output: CFlow,
    },
}

/// A BPMN choreography is the parallel composition of basic terms.
#[derive(Debug, Clone, Default)]
pub struct Choreography {
    pub elements: Vec<ChoreographyElement>,
}