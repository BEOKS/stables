use std::{collections::BTreeMap, path::PathBuf};

use crate::{content::ContentBlock, identity::SessionId};

/// Request to create a fresh interactive session.
///
/// This maps to ACP `session/new`. The runtime should allocate any session
/// state, attach requested MCP servers, and return a new [`SessionId`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSessionRequest {
    /// Absolute working directory that scopes relative file and terminal operations.
    pub cwd: PathBuf,
    /// MCP servers the runtime should connect to for this session.
    pub mcp_servers: Vec<McpServer>,
}

/// Response returned after a runtime creates a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSessionResponse {
    /// Runtime-assigned session identifier used by all future calls.
    pub session_id: SessionId,
    /// Optional initial mode state if the runtime supports modes.
    pub modes: Option<SessionModeState>,
    /// Initial configuration options exposed by the runtime.
    pub config_options: Vec<SessionConfigOption>,
}

/// Request to load an existing session and replay or restore its context.
///
/// This maps to ACP `session/load`. A runtime that supports this should restore
/// the session and emit relevant history through the event sink during the load
/// flow implemented by its adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadSessionRequest {
    /// Existing session to restore.
    pub session_id: SessionId,
    /// Working directory to associate with the restored session.
    pub cwd: PathBuf,
    /// MCP servers that should be available after loading.
    pub mcp_servers: Vec<McpServer>,
}

/// Response returned when an existing session has been loaded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoadSessionResponse {
    /// Restored mode state, if supported.
    pub modes: Option<SessionModeState>,
    /// Restored runtime configuration options.
    pub config_options: Vec<SessionConfigOption>,
}

/// Request to list sessions known to the runtime.
///
/// The result may represent persisted conversations, live sessions, or both,
/// depending on the concrete runtime adapter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListSessionsRequest {
    /// Optional working-directory filter.
    pub cwd: Option<PathBuf>,
    /// Opaque pagination cursor returned by a previous list response.
    pub cursor: Option<String>,
}

/// Page of sessions returned by `AgentRuntime::list_sessions`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionsResponse {
    /// Session metadata visible to Stables.
    pub sessions: Vec<SessionInfo>,
    /// Opaque cursor for the next page, if more sessions exist.
    pub next_cursor: Option<String>,
}

/// Lightweight metadata for a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    /// Stable session identifier.
    pub session_id: SessionId,
    /// Working directory associated with the session.
    pub cwd: PathBuf,
    /// Optional user-facing session title.
    pub title: Option<String>,
    /// Optional last-activity timestamp as provided by the runtime.
    pub updated_at: Option<String>,
}

/// MCP server configuration attached to a session.
///
/// Stables keeps this transport-neutral so ACP, MCP-native, and CLI adapters can
/// all share one session request model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpServer {
    /// Local subprocess MCP server connected over stdio.
    Stdio(McpStdioServer),
    /// Remote MCP server connected over HTTP.
    Http(McpHttpServer),
    /// Remote MCP server connected over server-sent events.
    Sse(McpHttpServer),
}

/// Local stdio MCP server launch configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpStdioServer {
    /// Human-readable server name.
    pub name: String,
    /// Executable path to launch.
    pub command: PathBuf,
    /// Command-line arguments passed to the executable.
    pub args: Vec<String>,
    /// Environment variables passed to the subprocess.
    pub env: BTreeMap<String, String>,
}

/// Remote HTTP or SSE MCP server connection configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpHttpServer {
    /// Human-readable server name.
    pub name: String,
    /// Server endpoint URL.
    pub url: String,
    /// Headers required by the remote MCP endpoint.
    pub headers: BTreeMap<String, String>,
}

/// Current session mode plus all modes the runtime exposes.
///
/// Modes are runtime-defined operating profiles such as ask, plan, code, or review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionModeState {
    /// Active mode for the session.
    pub current_mode_id: SessionModeId,
    /// Complete mode menu available to the host.
    pub available_modes: Vec<SessionMode>,
}

/// One runtime-defined operating mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMode {
    /// Stable mode identifier used in mode-switch requests.
    pub id: SessionModeId,
    /// User-facing mode name.
    pub name: String,
    /// Optional explanation of how this mode changes runtime behavior.
    pub description: Option<String>,
}

/// Identifier for a runtime-defined session mode.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionModeId(String);

impl SessionModeId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Request to switch a session into another runtime-defined mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetSessionModeRequest {
    /// Session whose mode should change.
    pub session_id: SessionId,
    /// Target mode. It should exist in [`SessionModeState::available_modes`].
    pub mode_id: SessionModeId,
}

/// Empty response indicating the session mode was accepted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetSessionModeResponse;

/// Runtime-defined session configuration option.
///
/// Config options are user-facing controls such as model selectors,
/// reasoning-level selectors, or boolean feature toggles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionConfigOption {
    /// Stable option identifier.
    pub id: SessionConfigId,
    /// User-facing option name.
    pub name: String,
    /// Optional help text shown near the control.
    pub description: Option<String>,
    /// Semantic category for UI placement and adapter logic.
    pub category: Option<SessionConfigCategory>,
    /// Shape and current value of this option.
    pub kind: SessionConfigKind,
}

/// Identifier for a runtime-defined session configuration option.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionConfigId(String);

impl SessionConfigId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Broad semantic category for a session configuration option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionConfigCategory {
    /// Option selects a session mode.
    Mode,
    /// Option selects a model.
    Model,
    /// Option selects a reasoning or thought budget level.
    ThoughtLevel,
    /// Runtime-specific category not known to Stables.
    Other(String),
}

/// Shape and current value of a session configuration option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionConfigKind {
    /// Single-value selector such as a model dropdown.
    Select {
        /// Currently selected option value.
        current_value: String,
        /// Values the user can choose.
        options: Vec<SessionConfigSelectOption>,
    },
    /// On/off toggle.
    Boolean {
        /// Current toggle value.
        current_value: bool,
    },
}

/// One selectable value for a [`SessionConfigKind::Select`] option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionConfigSelectOption {
    /// Runtime-defined value ID.
    pub value: String,
    /// User-facing value label.
    pub name: String,
    /// Optional value-specific explanation.
    pub description: Option<String>,
}

/// Request to update one session configuration option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetSessionConfigRequest {
    /// Session whose configuration should change.
    pub session_id: SessionId,
    /// Option to update.
    pub config_id: SessionConfigId,
    /// New option value.
    pub value: SessionConfigValue,
}

/// Value assigned to a session configuration option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionConfigValue {
    /// ID selected from a finite set of option values.
    Id(String),
    /// Boolean toggle value.
    Boolean(bool),
}

/// Response containing the authoritative option set after an update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetSessionConfigResponse {
    /// Complete set of options with their current values.
    pub config_options: Vec<SessionConfigOption>,
}

/// User prompt sent to a session.
///
/// This maps to ACP `session/prompt`. A runtime handles the prompt, emits
/// intermediate events, and returns when the turn stops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptRequest {
    /// Target session.
    pub session_id: SessionId,
    /// Structured prompt blocks from the user or host.
    pub prompt: Vec<ContentBlock>,
}

/// Final response for one prompt turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptResponse {
    /// Why the runtime stopped generating or acting.
    pub stop_reason: StopReason,
}

/// Reason one prompt turn ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// The runtime completed the turn normally.
    EndTurn,
    /// The runtime hit a token budget.
    MaxTokens,
    /// The runtime hit an allowed tool/request budget.
    MaxTurnRequests,
    /// The runtime refused to continue the turn.
    Refusal,
    /// The host or user cancelled the turn.
    Cancelled,
}

/// Request to cancel active work for a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelSessionRequest {
    /// Session whose in-flight work should stop.
    pub session_id: SessionId,
}
