use super::{AcpRuntimeConfig, AgentAuthScheme, AgentDefinitionId, AgentError, AgentProtocol};

/// Shared definition of an ACP-compatible agent runtime Stables can expose.
///
/// This type belongs in `stables-core` because the control plane, CLI, and
/// future stable-side workers all need the same vocabulary when they refer to
/// agents such as Codex, Claude Code, or custom local runners.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentDefinition {
    /// Stable key used by configs, logs, placement records, and API payloads.
    ///
    /// This should remain stable even if the display name changes.
    id: AgentDefinitionId,
    /// Human-readable title for lists, menus, and operator-facing messages.
    name: String,
    /// Human-readable explanation of what this agent is useful for.
    ///
    /// This helps users and placement logic understand the agent's role before
    /// more detailed capability modeling exists.
    description: String,
    supported_auth: Vec<AgentAuthScheme>,
    acp_runtime: AcpRuntimeConfig,
}

impl AgentDefinition {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        supported_auth: Vec<AgentAuthScheme>,
        acp_runtime: AcpRuntimeConfig,
    ) -> Result<Self, AgentError> {
        if supported_auth.is_empty() {
            return Err(AgentError::MissingAuthScheme);
        }

        Ok(Self {
            id: AgentDefinitionId::new(id)?,
            name: normalize_required(name, AgentError::MissingName)?,
            description: normalize_required(description, AgentError::MissingDescription)?,
            supported_auth,
            acp_runtime,
        })
    }

    pub fn id(&self) -> &AgentDefinitionId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn protocol(&self) -> AgentProtocol {
        AgentProtocol::Acp
    }

    pub fn supported_auth(&self) -> &[AgentAuthScheme] {
        &self.supported_auth
    }

    pub fn acp_runtime(&self) -> &AcpRuntimeConfig {
        &self.acp_runtime
    }
}

/// Backward-compatible alias while the domain migrates to explicit definitions.
pub type Agent = AgentDefinition;

fn normalize_required(value: impl Into<String>, error: AgentError) -> Result<String, AgentError> {
    let value = value.into().trim().to_owned();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}
