use super::AgentError;

/// Protocol Stables uses to talk to an external agent runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentProtocol {
    Acp,
}

/// Launch configuration for an ACP-compatible agent process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcpRuntimeConfig {
    command: String,
    args: Vec<String>,
}

impl AcpRuntimeConfig {
    pub fn new(
        command: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, AgentError> {
        Ok(Self {
            command: normalize_required(command, AgentError::MissingCommand)?,
            args: args
                .into_iter()
                .filter_map(|arg| {
                    let arg = arg.into().trim().to_owned();
                    if arg.is_empty() { None } else { Some(arg) }
                })
                .collect(),
        })
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &[String] {
        &self.args
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
