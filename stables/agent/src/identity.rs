use std::fmt;

/// Stable identifier for an interactive agent session.
///
/// In ACP this maps to `SessionId`. In Stables it is also the bridge point for
/// later control-plane identifiers such as job IDs, harness IDs, or stable IDs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(String);

impl SessionId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for SessionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SessionId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Protocol compatibility level understood by this Stables agent boundary.
///
/// This is not meant to mirror every upstream ACP release. It records the
/// version of the Stables-side contract an adapter negotiated with the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolVersion(u32);

impl ProtocolVersion {
    #[must_use]
    pub fn new(version: u32) -> Self {
        Self(version)
    }

    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }
}

/// Human-readable and diagnostic metadata for either side of the connection.
///
/// ACP calls this `Implementation`. Stables uses it for both agent runtimes and
/// hosts so logs, UI, metrics, and adapter debugging can identify participants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationInfo {
    /// Programmatic implementation name, for example `codex`, `claude-code`, or `stables`.
    pub name: String,
    /// Implementation version string reported by the runtime or host.
    pub version: String,
    /// Optional display title for UI surfaces.
    pub title: Option<String>,
}

impl ImplementationInfo {
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            title: None,
        }
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// One authentication path offered by an agent runtime.
///
/// The first version models ACP's stable `agent`-handled auth shape. More
/// specialized auth methods can be added later without changing the runtime trait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthMethod {
    /// Stable ID the host passes back in `AuthenticateRequest`.
    pub id: AuthMethodId,
    /// User-facing auth method name.
    pub name: String,
    /// Optional explanation shown before the user authenticates.
    pub description: Option<String>,
}

/// Identifier for an advertised authentication method.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthMethodId(String);

impl AuthMethodId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}
