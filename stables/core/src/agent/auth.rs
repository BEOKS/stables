use super::AgentError;

/// Authentication schemes an agent runtime definition can support.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentAuthScheme {
    ExistingCliSession,
    OAuth,
    ApiKey,
    None,
}

/// Secret storage reference for user-provided credentials.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretRef(String);

impl SecretRef {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentError> {
        Ok(Self(normalize_required(
            value,
            AgentError::MissingSecretRef,
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Authentication selected for a user-registered agent connection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegisteredAgentAuth {
    ExistingCliSession,
    OAuth { token: SecretRef },
    ApiKey { key: SecretRef },
    None,
}

fn normalize_required(value: impl Into<String>, error: AgentError) -> Result<String, AgentError> {
    let value = value.into().trim().to_owned();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}
