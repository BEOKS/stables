use std::{collections::BTreeMap, path::PathBuf};

use crate::identity::SessionId;

/// Request from a runtime to read a text file through the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadTextFileRequest {
    /// Session making the file request.
    pub session_id: SessionId,
    /// Absolute path to read.
    pub path: PathBuf,
    /// Optional one-based starting line.
    pub line: Option<u32>,
    /// Optional maximum number of lines.
    pub limit: Option<u32>,
}

/// Request from a runtime to write a text file through the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteTextFileRequest {
    /// Session making the file request.
    pub session_id: SessionId,
    /// Absolute path to write.
    pub path: PathBuf,
    /// Full text content to write.
    pub content: String,
}

/// Request from a runtime to create a host-managed terminal process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTerminalRequest {
    /// Session that owns the terminal.
    pub session_id: SessionId,
    /// Executable or shell command.
    pub command: String,
    /// Optional working directory override.
    pub cwd: Option<PathBuf>,
    /// Command-line arguments.
    pub args: Vec<String>,
    /// Environment variables for the process.
    pub env: BTreeMap<String, String>,
}

/// Request to read current terminal output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalOutputRequest {
    /// Session that owns the terminal.
    pub session_id: SessionId,
    /// Host-managed terminal ID.
    pub terminal_id: TerminalId,
}

/// Current output and optional exit state of a terminal process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalOutputResponse {
    /// Captured terminal output.
    pub output: String,
    /// Exit status if the command has already exited.
    pub exit_status: Option<ExitStatus>,
}

/// Portable process exit status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitStatus {
    /// Numeric exit code when the platform provides one.
    pub code: Option<i32>,
}

/// Response returned after the host creates a terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTerminalResponse {
    /// Host-managed terminal ID used by later terminal operations.
    pub terminal_id: TerminalId,
}

/// Request to release a terminal and its host resources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseTerminalRequest {
    /// Session that owns the terminal.
    pub session_id: SessionId,
    /// Terminal to release.
    pub terminal_id: TerminalId,
}

/// Request to kill a terminal process while preserving its ID for final output reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KillTerminalRequest {
    /// Session that owns the terminal.
    pub session_id: SessionId,
    /// Terminal process to kill.
    pub terminal_id: TerminalId,
}

/// Request to wait until a terminal process exits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitForTerminalExitRequest {
    /// Session that owns the terminal.
    pub session_id: SessionId,
    /// Terminal process to wait for.
    pub terminal_id: TerminalId,
}

/// Exit response for [`WaitForTerminalExitRequest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitForTerminalExitResponse {
    /// Final exit status.
    pub exit_status: ExitStatus,
}

/// Identifier for a host-managed terminal process.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerminalId(String);

impl TerminalId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}
