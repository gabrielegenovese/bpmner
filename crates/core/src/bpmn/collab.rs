use crate::bpmn::common::{CFlow};
use std::collections::HashSet;

/// BPMN Collaboration elements.
///
/// Corresponds exactly to the grammar:
///
/// start(e)
/// end(e)
/// task(e,e')  TODO: distinguish from Choreography
/// andSplit(e,E)
/// andJoin(E,e)
/// xorSplit(e,E)
/// xorJoin(E,e)
/// TODO: ADD UNIQUE COLLABORATION ELEMENTS (event based gateway, send event, receive event)
#[derive(Debug, Clone)]
pub enum CollaborationElement {
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
}

/// A BPMN Collaboration is the parallel composition of basic terms.
#[derive(Debug, Clone, Default)]
pub struct Collaboration {
    pub elements: Vec<CollaborationElement>,
}