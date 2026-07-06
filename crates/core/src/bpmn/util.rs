use crate::bpmn::common::{CFlow, CFlowState};

pub fn set_live(edges: &mut [CFlow]) {
    for edge in edges {
        edge.state = CFlowState::Live;
    }
}

pub fn set_dead(edges: &mut [CFlow]) {
    for edge in edges {
        edge.state = CFlowState::Dead;
    }
}

pub fn set_wait(edges: &mut [CFlow]) {
    for edge in edges {
        edge.state = CFlowState::Wait;
    }
}

/// True iff every edge is live.
pub fn is_live(edges: &[CFlow]) -> bool {
    edges.iter().all(|e| e.state == CFlowState::Live)
}

/// True iff every edge is dead.
pub fn is_dead(edges: &[CFlow]) -> bool {
    edges.iter().all(|e| e.state == CFlowState::Dead)
}

/// True iff at least one edge is waiting.
pub fn is_wait(edges: &[CFlow]) -> bool {
    edges.iter().any(|e| e.state == CFlowState::Wait)
}