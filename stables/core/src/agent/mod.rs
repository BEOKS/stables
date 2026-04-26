//! Agent runtime definitions, authentication, and user registrations.

mod auth;
mod catalog;
mod definition;
mod error;
mod id;
mod registered;
mod runtime;

pub use auth::{AgentAuthScheme, RegisteredAgentAuth, SecretRef};
pub use catalog::{
    AgentCatalog, CLAUDE_CODE_AGENT_ID, CODEX_AGENT_ID, HERMES_AGENT_ID, PI_AGENT_ID,
    built_in_agents,
};
pub use definition::{Agent, AgentDefinition};
pub use error::AgentError;
pub use id::{AgentDefinitionId, AgentId, RegisteredAgentId};
pub use registered::{RegisteredAgent, RegisteredAgentStatus};
pub use runtime::{AcpRuntimeConfig, AgentProtocol};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_definition_requires_an_id_name_description_auth_and_runtime() {
        assert!(matches!(
            AgentDefinition::new(
                " ",
                "codex",
                "local coding agent",
                vec![AgentAuthScheme::ExistingCliSession],
                AcpRuntimeConfig::new("codex-acp", Vec::<String>::new()).unwrap(),
            ),
            Err(AgentError::MissingId)
        ));

        assert!(matches!(
            AgentDefinition::new(
                "codex",
                " ",
                "local coding agent",
                vec![AgentAuthScheme::ExistingCliSession],
                AcpRuntimeConfig::new("codex-acp", Vec::<String>::new()).unwrap(),
            ),
            Err(AgentError::MissingName)
        ));

        assert!(matches!(
            AgentDefinition::new(
                "codex",
                "Codex",
                " ",
                vec![AgentAuthScheme::ExistingCliSession],
                AcpRuntimeConfig::new("codex-acp", Vec::<String>::new()).unwrap(),
            ),
            Err(AgentError::MissingDescription)
        ));

        assert!(matches!(
            AgentDefinition::new(
                "codex",
                "Codex",
                "local coding agent",
                Vec::new(),
                AcpRuntimeConfig::new("codex-acp", Vec::<String>::new()).unwrap(),
            ),
            Err(AgentError::MissingAuthScheme)
        ));
    }

    #[test]
    fn agent_definition_trims_identity_fields_and_exposes_acp_runtime() {
        let agent = AgentDefinition::new(
            " codex ",
            " Codex ",
            " Local coding agent ",
            vec![AgentAuthScheme::ExistingCliSession],
            AcpRuntimeConfig::new(" codex-acp ", vec![" --stdio "]).unwrap(),
        )
        .unwrap();

        assert_eq!(agent.id().as_str(), "codex");
        assert_eq!(agent.name(), "Codex");
        assert_eq!(agent.description(), "Local coding agent");
        assert_eq!(agent.protocol(), AgentProtocol::Acp);
        assert_eq!(agent.acp_runtime().command(), "codex-acp");
        assert_eq!(agent.acp_runtime().args(), &["--stdio"]);
        assert_eq!(
            agent.supported_auth(),
            &[AgentAuthScheme::ExistingCliSession]
        );
    }

    #[test]
    fn built_in_agents_define_acp_compatible_runtime_catalog() {
        let codex = AgentDefinition::codex();
        let claude_code = AgentDefinition::claude_code();
        let hermes = AgentDefinition::hermes();
        let pi = AgentDefinition::pi();

        assert_eq!(codex.id().as_str(), "codex");
        assert_eq!(codex.name(), "Codex");
        assert_eq!(codex.protocol(), AgentProtocol::Acp);
        assert!(codex.description().contains("OpenAI Codex"));
        assert!(
            codex
                .supported_auth()
                .contains(&AgentAuthScheme::ExistingCliSession)
        );

        assert_eq!(claude_code.id().as_str(), "claude-code");
        assert_eq!(claude_code.name(), "Claude Code");
        assert_eq!(claude_code.protocol(), AgentProtocol::Acp);
        assert!(claude_code.description().contains("Claude Code"));

        assert_eq!(hermes.id().as_str(), "hermes");
        assert!(hermes.supported_auth().contains(&AgentAuthScheme::ApiKey));

        assert_eq!(pi.id().as_str(), "pi");
        assert_eq!(pi.protocol(), AgentProtocol::Acp);
    }

    #[test]
    fn built_in_agent_catalog_finds_supported_agent_definitions() {
        let agents = built_in_agents();

        assert_eq!(
            agents.all(),
            &[
                AgentDefinition::codex(),
                AgentDefinition::claude_code(),
                AgentDefinition::hermes(),
                AgentDefinition::pi(),
            ]
        );
        assert_eq!(
            agents.find("claude-code"),
            Some(&AgentDefinition::claude_code())
        );
        assert_eq!(agents.find("unknown"), None);
    }

    #[test]
    fn registered_agent_records_user_selected_definition_auth_and_runtime() {
        let registered = RegisteredAgent::new(
            " my-codex ",
            AgentDefinition::codex().id().clone(),
            " Personal Codex ",
            RegisteredAgentAuth::ExistingCliSession,
            AcpRuntimeConfig::new(" codex-acp ", vec![" --stdio "]).unwrap(),
        )
        .unwrap();

        assert_eq!(registered.id().as_str(), "my-codex");
        assert_eq!(registered.definition_id().as_str(), "codex");
        assert_eq!(registered.display_name(), "Personal Codex");
        assert_eq!(registered.auth(), &RegisteredAgentAuth::ExistingCliSession);
        assert_eq!(registered.status(), RegisteredAgentStatus::Active);
        assert_eq!(registered.acp_runtime().command(), "codex-acp");
    }

    #[test]
    fn registered_agent_can_reference_secret_backed_auth() {
        let registered = RegisteredAgent::new(
            " hermes-work ",
            AgentDefinition::hermes().id().clone(),
            " Hermes Work ",
            RegisteredAgentAuth::ApiKey {
                key: SecretRef::new(" keychain:stables/hermes ").unwrap(),
            },
            AcpRuntimeConfig::new(" hermes-acp ", Vec::<String>::new()).unwrap(),
        )
        .unwrap();

        assert_eq!(
            registered.auth(),
            &RegisteredAgentAuth::ApiKey {
                key: SecretRef::new("keychain:stables/hermes").unwrap(),
            }
        );
    }
}
