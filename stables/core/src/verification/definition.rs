use crate::agent::AgentId;

use super::error::VerificationError;
use super::title::normalize_verification_title;

/// A concrete way to decide whether a goal has been satisfied.
///
/// Command verification is for deterministic checks such as Python, Node, or
/// shell scripts. Agent verification is for qualitative checks that need an
/// agent to judge the result instead of a deterministic script.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verification {
    /// Deterministic verification by running a command on the selected stable.
    Command {
        /// Human-readable check title shown when Stables reports verification.
        ///
        /// The title is normalized to start with `Verify` because it represents
        /// an actual check Stables performs to determine whether the goal has
        /// been satisfied.
        title: String,
        /// Command line executed by the stable-side worker.
        ///
        /// This should be used for static or reproducible checks such as
        /// Python scripts, Node scripts, shell commands, linters, tests, or
        /// custom validators.
        command: String,
    },
    /// Qualitative verification delegated to an agent.
    Agent {
        /// Human-readable check title shown when Stables reports verification.
        ///
        /// The title is normalized to start with `Verify` because it represents
        /// an actual check Stables performs to determine whether the goal has
        /// been satisfied.
        title: String,
        /// Stable identifier of the agent that should perform the verification.
        agent_id: AgentId,
        /// Prompt sent to the verifying agent.
        ///
        /// This should describe the judgment rubric for cases that are hard to
        /// validate with deterministic scripts, such as UX quality, writing
        /// quality, or subjective requirement satisfaction.
        prompt: String,
    },
}

impl Verification {
    pub fn command(
        title: impl Into<String>,
        command: impl Into<String>,
    ) -> Result<Self, VerificationError> {
        Ok(Self::Command {
            title: normalize_verification_title(title)?,
            command: normalize_required(command, VerificationError::MissingCommand)?,
        })
    }

    pub fn agent(
        title: impl Into<String>,
        agent_id: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Result<Self, VerificationError> {
        Ok(Self::Agent {
            title: normalize_verification_title(title)?,
            agent_id: AgentId::new(agent_id).map_err(|_| VerificationError::MissingAgent)?,
            prompt: normalize_required(prompt, VerificationError::MissingAgentPrompt)?,
        })
    }
}

fn normalize_required(
    value: impl Into<String>,
    error: VerificationError,
) -> Result<String, VerificationError> {
    let value = value.into().trim().to_owned();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}
