/// Validation failures for verification construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationError {
    MissingTitle,
    MissingCommand,
    MissingAgent,
    MissingAgentPrompt,
}
