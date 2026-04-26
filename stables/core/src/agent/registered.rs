use super::{
    AcpRuntimeConfig, AgentDefinitionId, AgentError, RegisteredAgentAuth, RegisteredAgentId,
};

/// Current usability state for a user-registered agent connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisteredAgentStatus {
    Active,
    NeedsReauth,
    Disabled,
}

/// A concrete agent connection the user has authenticated and can run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredAgent {
    id: RegisteredAgentId,
    definition_id: AgentDefinitionId,
    display_name: String,
    auth: RegisteredAgentAuth,
    status: RegisteredAgentStatus,
    acp_runtime: AcpRuntimeConfig,
}

impl RegisteredAgent {
    pub fn new(
        id: impl Into<String>,
        definition_id: AgentDefinitionId,
        display_name: impl Into<String>,
        auth: RegisteredAgentAuth,
        acp_runtime: AcpRuntimeConfig,
    ) -> Result<Self, AgentError> {
        Ok(Self {
            id: RegisteredAgentId::new(id)?,
            definition_id,
            display_name: normalize_required(display_name, AgentError::MissingDisplayName)?,
            auth,
            status: RegisteredAgentStatus::Active,
            acp_runtime,
        })
    }

    pub fn id(&self) -> &RegisteredAgentId {
        &self.id
    }

    pub fn definition_id(&self) -> &AgentDefinitionId {
        &self.definition_id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn auth(&self) -> &RegisteredAgentAuth {
        &self.auth
    }

    pub fn status(&self) -> RegisteredAgentStatus {
        self.status
    }

    pub fn acp_runtime(&self) -> &AcpRuntimeConfig {
        &self.acp_runtime
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
