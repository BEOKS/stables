use crate::terminal::TerminalId;

/// Runtime command that can be invoked by the host or user.
///
/// This is intentionally minimal for now; structured command input can be added
/// after the first adapter needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableCommand {
    /// Stable command name.
    pub name: String,
    /// User-facing command description.
    pub description: String,
}

/// Tool call started by the runtime.
///
/// Tool calls are operations the runtime performs or asks the host to perform:
/// file reads, edits, searches, terminal commands, network calls, or custom actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    /// Stable ID used to correlate later [`ToolCallUpdate`] values.
    pub id: ToolCallId,
    /// User-facing title describing the operation.
    pub title: String,
    /// Broad category used by UI and policy layers.
    pub kind: ToolKind,
    /// Current execution state.
    pub status: ToolCallStatus,
    /// Human-readable or structured output produced by the tool call.
    pub content: Vec<ToolCallContent>,
}

/// Partial update for an existing [`ToolCall`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallUpdate {
    /// Tool call being updated.
    pub id: ToolCallId,
    /// Optional replacement title.
    pub title: Option<String>,
    /// Optional replacement status.
    pub status: Option<ToolCallStatus>,
    /// Replacement visible content, if any.
    pub content: Vec<ToolCallContent>,
}

/// Identifier for a tool call inside one session.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolCallId(String);

impl ToolCallId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Broad category of tool operation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToolKind {
    /// Read-only file or resource access.
    Read,
    /// File or state mutation.
    Edit,
    /// Command or process execution.
    Execute,
    /// Search over files, documents, logs, or external indexes.
    Search,
    /// Runtime-specific category not yet modeled by Stables.
    #[default]
    Other,
}

/// Execution state of a tool call.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToolCallStatus {
    /// Tool call has been announced but not started.
    #[default]
    Pending,
    /// Tool call is currently running.
    Running,
    /// Tool call finished successfully.
    Completed,
    /// Tool call failed.
    Failed,
}

/// Visible content associated with a tool call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolCallContent {
    /// Text output, logs, summaries, or errors.
    Text(String),
    /// Reference to a host-managed terminal.
    Terminal { terminal_id: TerminalId },
}
