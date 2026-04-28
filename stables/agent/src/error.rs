use std::{error::Error, fmt, future::Future, pin::Pin};

/// Result type shared by all agent-domain operations.
pub type AgentResult<T> = Result<T, AgentError>;

/// Boxed async return type used to keep crate traits object-safe.
///
/// The project can later replace this with native async trait methods if the
/// MSRV and object-safety story make that desirable.
pub type AgentFuture<'a, T> = Pin<Box<dyn Future<Output = AgentResult<T>> + Send + 'a>>;

/// Error vocabulary for agent-runtime and host-capability failures.
///
/// This enum is deliberately small. Adapters should translate provider-specific
/// failures into these variants at the boundary, and may preserve detailed text
/// in [`AgentError::InvalidRequest`] or [`AgentError::Runtime`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentError {
    /// The runtime or host does not implement the requested domain operation.
    Unsupported { operation: &'static str },
    /// The caller supplied a malformed or semantically invalid request.
    InvalidRequest { message: String },
    /// The underlying runtime, provider, process, or host operation failed.
    Runtime { message: String },
    /// The operation was intentionally cancelled by the caller or user.
    Cancelled,
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { operation } => {
                write!(f, "agent operation is not supported: {operation}")
            }
            Self::InvalidRequest { message } => write!(f, "invalid agent request: {message}"),
            Self::Runtime { message } => write!(f, "agent runtime error: {message}"),
            Self::Cancelled => f.write_str("agent operation was cancelled"),
        }
    }
}

impl Error for AgentError {}

pub(crate) fn unsupported<T>(operation: &'static str) -> AgentFuture<'static, T> {
    Box::pin(async move { Err(AgentError::Unsupported { operation }) })
}
