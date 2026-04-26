/// Validation failures for agent construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentError {
    MissingId,
    MissingName,
    MissingDescription,
    MissingAuthScheme,
    MissingCommand,
    MissingSecretRef,
    MissingDisplayName,
}
