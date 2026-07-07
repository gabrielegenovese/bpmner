pub type ControlFlow = String;

impl ControlFlow {
    pub fn new(id: impl Into<ControlFlow>) -> Self {
        Self {
            id: id.into()
        }
    }
}

pub type MessageFlow = String;

impl MessageFlow {
    pub fn new(id: impl Into<MessageFlow>) -> Self {
        Self {
            id: id.into()
        }
    }
}
