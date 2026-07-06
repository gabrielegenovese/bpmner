
pub type CFlowId = String;

/// CFlow status according to the operational semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CFlowState {
    Wait,
    Live,
    Dead,
}

/// CFlow of a BPMN choreography.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CFlow {
    pub id: CFlowId,
    pub state: CFlowState,
}

impl CFlow {
    pub fn new(id: impl Into<CFlowId>) -> Self {
        Self {
            id: id.into(),
            state: CFlowState::Wait,
        }
    }
}