use crate::goal::Goal;
use crate::runtime::{Runtime, RuntimeError};

/// The smallest runnable unit Stables can place onto a stable.
///
/// A harness binds user intent (`Goal`) to an execution environment (`Runtime`).
/// Scheduling, leases, artifacts, and retries deliberately live outside this
/// type so the harness remains portable across local, cloud, and on-premise
/// stables.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Harness {
    /// Human-readable title for this harness.
    ///
    /// This is intended for CLI listings, UI labels, logs, and operator-facing
    /// diagnostics, not as a stable machine identifier.
    name: String,
    /// Environment the stable-side worker uses to run this harness.
    ///
    /// Runtime describes where the harness runs. Scheduling, retry policy, and
    /// artifact handling are intentionally kept outside this field.
    runtime: Runtime,
    /// Goal this harness is expected to satisfy.
    ///
    /// The harness owns a goal so the execution contract always carries both
    /// intent and verification methods together.
    goal: Goal,
}

impl Harness {
    pub fn new(
        name: impl Into<String>,
        runtime: Runtime,
        goal: Goal,
    ) -> Result<Self, HarnessError> {
        let name = normalize_required(name, HarnessError::MissingName)?;
        let runtime = runtime.normalized().map_err(HarnessError::InvalidRuntime)?;

        Ok(Self {
            name,
            runtime,
            goal,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn goal(&self) -> &Goal {
        &self.goal
    }
}

/// Validation failures for harness construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HarnessError {
    MissingName,
    InvalidRuntime(RuntimeError),
}

fn normalize_required(
    value: impl Into<String>,
    error: HarnessError,
) -> Result<String, HarnessError> {
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
    use crate::goal::Goal;
    use crate::runtime::Runtime;
    use crate::verification::Verification;

    #[test]
    fn harness_requires_a_name_runtime_and_goal() {
        let goal = Goal::new(
            "demo",
            "ship a demo",
            vec![Verification::command("tests pass", "cargo test").unwrap()],
        )
        .unwrap();

        assert!(matches!(
            Harness::new(" ", Runtime::local("cargo test").unwrap(), goal),
            Err(HarnessError::MissingName)
        ));
    }

    #[test]
    fn harness_keeps_the_minimum_execution_contract() {
        let goal = Goal::new(
            "demo",
            "ship a demo",
            vec![Verification::command("tests pass", "cargo test").unwrap()],
        )
        .unwrap();

        let harness = Harness::new(
            " local demo ",
            Runtime::local(" cargo test ").unwrap(),
            goal,
        )
        .unwrap();

        assert_eq!(harness.name(), "local demo");
        assert_eq!(
            harness.runtime(),
            &Runtime::Local {
                command: "cargo test".into(),
            }
        );
        assert_eq!(harness.goal().description(), "ship a demo");
    }
}
