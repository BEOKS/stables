use crate::verification::Verification;

/// The user-facing intent that a harness is expected to satisfy.
///
/// A goal is intentionally smaller than a full job: it captures what the user
/// wants and the verifiable mechanisms Stables can use before deciding where,
/// when, and how to run the harness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Goal {
    /// Stable key used by harness specs, logs, result records, and UI anchors
    /// to refer to this goal without depending on its human-readable wording.
    id: GoalId,
    /// Human-readable explanation of what should become true.
    ///
    /// This is the main text shown to users and agents when they need to
    /// understand the goal in natural language.
    description: String,
    /// Verification methods Stables can execute or delegate to decide whether
    /// the goal is satisfied.
    ///
    /// At least one verification is required so a goal never remains a purely
    /// aspirational statement.
    verifications: Vec<Verification>,
}

impl Goal {
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        verifications: Vec<Verification>,
    ) -> Result<Self, GoalError> {
        let id = GoalId::new(id)?;
        let description = normalize_required(description, GoalError::MissingDescription)?;

        if verifications.is_empty() {
            return Err(GoalError::MissingVerification);
        }

        Ok(Self {
            id,
            description,
            verifications,
        })
    }

    pub fn id(&self) -> &GoalId {
        &self.id
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn verifications(&self) -> &[Verification] {
        &self.verifications
    }
}

/// A stable identifier for a goal inside a harness definition.
///
/// The identifier is separate from the description so UI, logs, persisted
/// results, and future retries can refer to the same goal even if the wording is
/// edited later.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalId(String);

impl GoalId {
    pub fn new(value: impl Into<String>) -> Result<Self, GoalError> {
        Ok(Self(normalize_required(value, GoalError::MissingId)?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validation failures for goal construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoalError {
    MissingId,
    MissingDescription,
    MissingVerification,
}

fn normalize_required(value: impl Into<String>, error: GoalError) -> Result<String, GoalError> {
    let value = value.into().trim().to_owned();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentId;

    #[test]
    fn goal_requires_an_id_description_and_verification() {
        assert!(matches!(
            Goal::new(
                " ",
                "ship a demo",
                vec![Verification::command("tests pass", "cargo test").unwrap()]
            ),
            Err(GoalError::MissingId)
        ));

        assert!(matches!(
            Goal::new(
                "demo",
                " ",
                vec![Verification::command("tests pass", "cargo test").unwrap()]
            ),
            Err(GoalError::MissingDescription)
        ));

        assert!(matches!(
            Goal::new("demo", "ship a demo", Vec::new()),
            Err(GoalError::MissingVerification)
        ));
    }

    #[test]
    fn goal_trims_id_description_and_verification_inputs() {
        let goal = Goal::new(
            " demo ",
            " ship a demo ",
            vec![
                Verification::command(" tests pass ", " cargo test ").unwrap(),
                Verification::agent(
                    "Verify qualitative review",
                    " codex ",
                    " judge whether the implementation satisfies the goal ",
                )
                .unwrap(),
            ],
        )
        .unwrap();

        assert_eq!(goal.id().as_str(), "demo");
        assert_eq!(goal.description(), "ship a demo");
        assert_eq!(
            goal.verifications(),
            &[
                Verification::Command {
                    title: "Verify tests pass".into(),
                    command: "cargo test".into()
                },
                Verification::Agent {
                    title: "Verify qualitative review".into(),
                    agent_id: AgentId::new("codex").unwrap(),
                    prompt: "judge whether the implementation satisfies the goal".into()
                }
            ]
        );
    }
}
