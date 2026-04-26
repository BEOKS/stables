//! Shared domain types and rules for Stables.

pub mod agent;
pub mod goal;
pub mod harness;
pub mod runtime;
pub mod verification;

pub use agent::{
    AcpRuntimeConfig, Agent, AgentAuthScheme, AgentCatalog, AgentDefinition, AgentDefinitionId,
    AgentError, AgentId, AgentProtocol, CLAUDE_CODE_AGENT_ID, CODEX_AGENT_ID, HERMES_AGENT_ID,
    PI_AGENT_ID, RegisteredAgent, RegisteredAgentAuth, RegisteredAgentId, RegisteredAgentStatus,
    SecretRef, built_in_agents,
};
pub use goal::{Goal, GoalError, GoalId};
pub use harness::{Harness, HarnessError};
pub use runtime::{Runtime, RuntimeError};
pub use verification::{Verification, VerificationError};
