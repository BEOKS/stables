use super::{AcpRuntimeConfig, AgentAuthScheme, AgentDefinition};

pub const CODEX_AGENT_ID: &str = "codex";
pub const CLAUDE_CODE_AGENT_ID: &str = "claude-code";
pub const HERMES_AGENT_ID: &str = "hermes";
pub const PI_AGENT_ID: &str = "pi";

impl AgentDefinition {
    /// Built-in definition for the OpenAI Codex local coding agent.
    pub fn codex() -> Self {
        AgentDefinition::new(
            CODEX_AGENT_ID,
            "Codex",
            "OpenAI Codex agent for local repository implementation, debugging, and verification work.",
            vec![
                AgentAuthScheme::ExistingCliSession,
                AgentAuthScheme::OAuth,
                AgentAuthScheme::ApiKey,
            ],
            AcpRuntimeConfig::new("codex-acp", Vec::<String>::new()).unwrap(),
        )
        .expect("built-in Codex agent definition must be valid")
    }

    /// Built-in definition for Anthropic Claude Code.
    pub fn claude_code() -> Self {
        AgentDefinition::new(
            CLAUDE_CODE_AGENT_ID,
            "Claude Code",
            "Claude Code agent for local repository implementation, debugging, and verification work.",
            vec![AgentAuthScheme::ExistingCliSession, AgentAuthScheme::OAuth],
            AcpRuntimeConfig::new("claude-agent-acp", Vec::<String>::new()).unwrap(),
        )
        .expect("built-in Claude Code agent definition must be valid")
    }

    /// Built-in definition for Hermes Agent.
    pub fn hermes() -> Self {
        AgentDefinition::new(
            HERMES_AGENT_ID,
            "Hermes",
            "Hermes ACP-compatible agent runtime for research and coding workflows.",
            vec![AgentAuthScheme::ApiKey, AgentAuthScheme::OAuth],
            AcpRuntimeConfig::new("hermes-acp", Vec::<String>::new()).unwrap(),
        )
        .expect("built-in Hermes agent definition must be valid")
    }

    /// Built-in definition for Pi Agent.
    pub fn pi() -> Self {
        AgentDefinition::new(
            PI_AGENT_ID,
            "Pi Agent",
            "Pi ACP-compatible coding agent runtime for local task execution.",
            vec![AgentAuthScheme::ApiKey, AgentAuthScheme::OAuth],
            AcpRuntimeConfig::new("pi-acp", Vec::<String>::new()).unwrap(),
        )
        .expect("built-in Pi agent definition must be valid")
    }
}

/// Catalog of agent runtime definitions Stables can offer to users.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCatalog {
    agents: Vec<AgentDefinition>,
}

impl AgentCatalog {
    pub fn built_in() -> Self {
        Self {
            agents: vec![
                AgentDefinition::codex(),
                AgentDefinition::claude_code(),
                AgentDefinition::hermes(),
                AgentDefinition::pi(),
            ],
        }
    }

    pub fn all(&self) -> &[AgentDefinition] {
        &self.agents
    }

    pub fn find(&self, id: &str) -> Option<&AgentDefinition> {
        self.agents.iter().find(|agent| agent.id().as_str() == id)
    }
}

/// Agent implementations Stables supports out of the box.
pub fn built_in_agents() -> AgentCatalog {
    AgentCatalog::built_in()
}
