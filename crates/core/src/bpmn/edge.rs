#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControlFlow {
    id: String,
}

impl ControlFlow {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageFlow {
    id: String,
}

impl MessageFlow {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
