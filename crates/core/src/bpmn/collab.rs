use crate::bpmn::common::{ControlFlow};
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
        inputs: HashSet<ControlFlow>,
        output: ControlFlow,
    },
    XorSplit {
        input: ControlFlow,
        output: HashSet<ControlFlow>,
    },
    XorJoin {
        inputs: HashSet<ControlFlow>,
        output: ControlFlow,
    },
}

/// A BPMN Collaboration is the parallel composition of basic terms.
#[derive(Debug, Clone, Default)]
pub struct Collaboration {
    pub elements: Vec<CollaborationElement>,
}