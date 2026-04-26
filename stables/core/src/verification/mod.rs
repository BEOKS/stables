//! Verification checks used by goals.

mod definition;
mod error;
mod title;

pub use definition::Verification;
pub use error::VerificationError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentId;

    #[test]
    fn verification_trims_inputs_and_normalizes_titles() {
        assert_eq!(
            Verification::command(" tests pass ", " cargo test ").unwrap(),
            Verification::Command {
                title: "Verify tests pass".into(),
                command: "cargo test".into(),
            }
        );

        assert_eq!(
            Verification::agent(
                "Verify qualitative review",
                " codex ",
                " judge whether the implementation satisfies the goal ",
            )
            .unwrap(),
            Verification::Agent {
                title: "Verify qualitative review".into(),
                agent_id: AgentId::new("codex").unwrap(),
                prompt: "judge whether the implementation satisfies the goal".into(),
            }
        );
    }

    #[test]
    fn verification_requires_a_non_empty_title_command_agent_or_prompt() {
        assert!(matches!(
            Verification::command(" ", "cargo test"),
            Err(VerificationError::MissingTitle)
        ));

        assert!(matches!(
            Verification::command("tests pass", " "),
            Err(VerificationError::MissingCommand)
        ));

        assert!(matches!(
            Verification::agent("review quality", " ", "judge the result"),
            Err(VerificationError::MissingAgent)
        ));

        assert!(matches!(
            Verification::agent("review quality", "codex", " "),
            Err(VerificationError::MissingAgentPrompt)
        ));
    }
}
