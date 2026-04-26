/// The environment where a harness runs.
///
/// Runtime names the execution environment, not just the launch command. A
/// harness can run on the local machine or a registered remote environment
/// while still carrying the command used inside that environment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Runtime {
    /// Run the harness on the local machine.
    Local { command: String },
    /// Run the harness on a registered remote environment.
    Remote { target: String, command: String },
}

impl Runtime {
    pub fn local(command: impl Into<String>) -> Result<Self, RuntimeError> {
        Ok(Self::Local {
            command: normalize_required(command, RuntimeError::MissingCommand)?,
        })
    }

    pub fn remote(
        target: impl Into<String>,
        command: impl Into<String>,
    ) -> Result<Self, RuntimeError> {
        Ok(Self::Remote {
            target: normalize_required(target, RuntimeError::MissingTarget)?,
            command: normalize_required(command, RuntimeError::MissingCommand)?,
        })
    }

    pub(crate) fn normalized(self) -> Result<Self, RuntimeError> {
        match self {
            Self::Local { command } => Self::local(command),
            Self::Remote { target, command } => Self::remote(target, command),
        }
    }
}

/// Validation failures for runtime construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    MissingCommand,
    MissingTarget,
}

fn normalize_required(
    value: impl Into<String>,
    error: RuntimeError,
) -> Result<String, RuntimeError> {
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

    #[test]
    fn local_runtime_requires_a_command() {
        assert_eq!(
            Runtime::local(" cargo test ").unwrap(),
            Runtime::Local {
                command: "cargo test".into(),
            }
        );

        assert!(matches!(
            Runtime::local(" "),
            Err(RuntimeError::MissingCommand)
        ));
    }

    #[test]
    fn remote_runtime_requires_a_target_and_command() {
        assert_eq!(
            Runtime::remote(" ci-us-east ", " cargo test ").unwrap(),
            Runtime::Remote {
                target: "ci-us-east".into(),
                command: "cargo test".into(),
            }
        );

        assert!(matches!(
            Runtime::remote(" ", "cargo test"),
            Err(RuntimeError::MissingTarget)
        ));
        assert!(matches!(
            Runtime::remote("buildbox-1", " "),
            Err(RuntimeError::MissingCommand)
        ));
    }
}
