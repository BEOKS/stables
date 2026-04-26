use super::AgentError;

/// Stable identifier for an agent runtime definition known to Stables.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentDefinitionId(String);

impl AgentDefinitionId {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentError> {
        Ok(Self(normalize_required(value, AgentError::MissingId)?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Backward-compatible alias for agent definition identifiers.
pub type AgentId = AgentDefinitionId;

/// Stable identifier for a user-registered agent connection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredAgentId(String);

impl RegisteredAgentId {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentError> {
        Ok(Self(normalize_required(value, AgentError::MissingId)?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn normalize_required(value: impl Into<String>, error: AgentError) -> Result<String, AgentError> {
    let value = value.into().trim().to_owned();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}
