use crate::{identity::SessionId, tool::ToolCallUpdate};

/// Request from the runtime asking the host/user to authorize a tool action.
///
/// This maps to ACP `session/request_permission`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionRequest {
    /// Session that owns the tool call.
    pub session_id: SessionId,
    /// Tool call requiring approval.
    pub tool_call: ToolCallUpdate,
    /// Choices the host should present to the user or policy engine.
    pub options: Vec<PermissionOption>,
}

/// Host response to a [`PermissionRequest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionResponse {
    /// User or policy decision.
    pub outcome: PermissionOutcome,
}

/// Result of a permission prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionOutcome {
    /// Prompt turn was cancelled before a decision was made.
    Cancelled,
    /// One of the provided options was selected.
    Selected { option_id: PermissionOptionId },
}

/// One choice shown for a permission request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionOption {
    /// Stable option ID returned if selected.
    pub id: PermissionOptionId,
    /// User-facing label.
    pub name: String,
    /// Semantic effect of the option.
    pub kind: PermissionOptionKind,
}

/// Identifier for one permission option.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PermissionOptionId(String);

impl PermissionOptionId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Semantic kind of permission decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionOptionKind {
    /// Allow only this operation.
    AllowOnce,
    /// Allow this and similar future operations.
    AllowAlways,
    /// Reject only this operation.
    RejectOnce,
    /// Reject this and similar future operations.
    RejectAlways,
}
