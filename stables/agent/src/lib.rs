//! Agent runtime contracts for Stables.
//!
//! This module defines Stables' internal agent domain model. It is intentionally
//! inspired by Agent Client Protocol (ACP), but it does not expose ACP transport,
//! JSON-RPC method names, or wire-format details to the rest of Stables.
//!
//! Domain glossary:
//! - **Agent runtime**: a concrete coding agent implementation such as Codex,
//!   Claude Code, OpenCode, or an ACP-compatible process.
//! - **Host**: the client-side environment that lends capabilities to the
//!   runtime, such as file access, terminal execution, and permission prompts.
//! - **Session**: one interactive conversation/work context with its own working
//!   directory, MCP servers, mode/config state, and streamed events.
//! - **Prompt turn**: one user message sent into a session, ending with a
//!   [`StopReason`].
//! - **Event sink**: the Stables-facing equivalent of ACP `session/update`;
//!   runtimes publish message chunks, tool calls, plans, and session metadata
//!   through it.
//!
//! ACP maps cleanly to three Stables boundaries:
//! - [`AgentRuntime`]: client/control-plane requests sent to a coding agent.
//! - [`AgentEventSink`]: streamed runtime updates emitted by the agent.
//! - [`AgentHost`]: host capabilities the agent may call back into.

use std::{collections::BTreeMap, error::Error, fmt, future::Future, path::PathBuf, pin::Pin};

/// Result type shared by all agent-domain operations.
pub type AgentResult<T> = Result<T, AgentError>;

/// Boxed async return type used to keep [`AgentRuntime`] and [`AgentHost`] object-safe.
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

fn unsupported<T>(operation: &'static str) -> AgentFuture<'static, T> {
    Box::pin(async move { Err(AgentError::Unsupported { operation }) })
}

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

/// Initial handshake request from Stables or a client host to an agent runtime.
///
/// This corresponds to ACP `initialize`. It establishes the contract version,
/// identifies the host, and advertises which host-side callbacks the runtime may use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializeRequest {
    /// Highest Stables agent contract version supported by the caller.
    pub protocol_version: ProtocolVersion,
    /// Identity of the host/client/control-plane side.
    pub host: ImplementationInfo,
    /// Capabilities that the host is willing to expose to the agent runtime.
    pub host_capabilities: HostCapabilities,
}

/// Initial handshake response from an agent runtime.
///
/// This is the Stables-side equivalent of ACP `InitializeResponse`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializeResponse {
    /// Contract version selected by the runtime.
    pub protocol_version: ProtocolVersion,
    /// Identity of the concrete agent runtime.
    pub agent: ImplementationInfo,
    /// Agent-side features available after initialization.
    pub capabilities: AgentCapabilities,
    /// Authentication methods the runtime requires before session creation.
    pub auth_methods: Vec<AuthMethod>,
}

/// One authentication path offered by an agent runtime.
///
/// The first version models ACP's stable `agent`-handled auth shape. More
/// specialized auth methods can be added later without changing the runtime trait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthMethod {
    /// Stable ID the host passes back in [`AuthenticateRequest`].
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

/// Request to complete one of the auth methods returned during initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticateRequest {
    /// Auth method selected by the host/user.
    pub method_id: AuthMethodId,
}

/// Empty response indicating authentication completed successfully.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthenticateResponse;

/// Feature set implemented by the agent runtime.
///
/// This is the agent side of capability negotiation. It tells Stables which
/// optional runtime operations are legal to call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentCapabilities {
    /// Whether [`AgentRuntime::load_session`] can restore an existing session.
    pub load_session: bool,
    /// Prompt input types the runtime can accept.
    pub prompt: PromptCapabilities,
    /// MCP server transports the runtime can attach to sessions.
    pub mcp: McpCapabilities,
    /// Optional session-management features supported by the runtime.
    pub sessions: SessionCapabilities,
}

/// Prompt content types accepted by a runtime.
///
/// Text and resource links are considered baseline. These flags describe
/// additional prompt payload types the host may include.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromptCapabilities {
    /// Runtime can process image content blocks.
    pub image: bool,
    /// Runtime can process audio content blocks.
    pub audio: bool,
    /// Runtime can consume embedded resource contents directly in prompts.
    pub embedded_context: bool,
}

/// MCP transport support exposed by an agent runtime.
///
/// Stdio MCP servers are treated as baseline. HTTP and SSE require explicit support.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct McpCapabilities {
    /// Runtime can connect to HTTP MCP servers.
    pub http: bool,
    /// Runtime can connect to SSE MCP servers.
    pub sse: bool,
}

/// Optional session lifecycle and session-state features.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionCapabilities {
    /// Runtime can list persisted or known sessions.
    pub list: bool,
    /// Runtime can fork a session into an independent child context.
    pub fork: bool,
    /// Runtime can resume an existing session without replaying history.
    pub resume: bool,
    /// Runtime can explicitly close and release session resources.
    pub close: bool,
    /// Runtime supports user-selectable operating modes.
    pub modes: bool,
    /// Runtime supports session configuration selectors or toggles.
    pub config_options: bool,
}

/// Capabilities exposed by the host/client side to an agent runtime.
///
/// This is the Stables counterpart to ACP `ClientCapabilities`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostCapabilities {
    /// File-system operations the host allows the runtime to request.
    pub filesystem: FileSystemCapabilities,
    /// Whether terminal creation/output/kill/release callbacks are available.
    pub terminal: bool,
    /// Whether the runtime may request explicit user permission for tool calls.
    pub permissions: bool,
}

/// File-system operations available through [`AgentHost`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileSystemCapabilities {
    /// Runtime may ask the host to read text files.
    pub read_text_file: bool,
    /// Runtime may ask the host to write text files.
    pub write_text_file: bool,
}

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
/// the session and emit relevant history through [`AgentEventSink`] during the
/// load flow implemented by its adapter.
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

/// Page of sessions returned by [`AgentRuntime::list_sessions`].
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
/// intermediate [`AgentEvent`] values, and returns when the turn stops.
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

/// Structured content exchanged in prompts, model output, and tool results.
///
/// This mirrors ACP/MCP content blocks while remaining independent from either
/// protocol's exact JSON representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentBlock {
    /// Plain text or Markdown text.
    Text(String),
    /// Base64-encoded image content plus MIME type and optional source URI.
    Image {
        data: String,
        mime_type: String,
        uri: Option<String>,
    },
    /// Base64-encoded audio content plus MIME type.
    Audio { data: String, mime_type: String },
    /// Link to a resource the runtime can fetch or interpret.
    ResourceLink {
        uri: String,
        name: Option<String>,
        mime_type: Option<String>,
    },
    /// Fully embedded text resource used when the runtime should not fetch it.
    Resource {
        uri: String,
        mime_type: String,
        text: String,
    },
}

impl ContentBlock {
    /// Convenience constructor for a text content block.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }
}

/// Runtime event emitted while a session or prompt turn is in progress.
///
/// This is Stables' canonical form of ACP `session/update`. Adapters should
/// translate native provider streams into this enum before the control plane or
/// UI sees them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    /// Echo or replay of user-message content.
    UserMessageChunk {
        session_id: SessionId,
        content: ContentBlock,
    },
    /// Visible assistant/model output.
    AgentMessageChunk {
        session_id: SessionId,
        content: ContentBlock,
    },
    /// Internal reasoning or progress text that a runtime chooses to expose.
    AgentThoughtChunk {
        session_id: SessionId,
        content: ContentBlock,
    },
    /// New tool call started by the runtime.
    ToolCall {
        session_id: SessionId,
        tool_call: ToolCall,
    },
    /// Partial update for an existing tool call.
    ToolCallUpdate {
        session_id: SessionId,
        update: ToolCallUpdate,
    },
    /// Complete current plan for the session or turn.
    Plan { session_id: SessionId, plan: Plan },
    /// Runtime-provided slash commands or command palette entries changed.
    AvailableCommands {
        session_id: SessionId,
        commands: Vec<AvailableCommand>,
    },
    /// Active session mode changed.
    CurrentMode {
        session_id: SessionId,
        mode_id: SessionModeId,
    },
    /// Session configuration controls changed.
    ConfigOptions {
        session_id: SessionId,
        config_options: Vec<SessionConfigOption>,
    },
    /// Session metadata such as title or last-updated timestamp changed.
    SessionInfo {
        session_id: SessionId,
        title: Option<String>,
        updated_at: Option<String>,
    },
}

/// Receiver for streamed runtime events.
///
/// The sink is passed into prompt/session operations so adapters can forward
/// updates immediately without coupling runtimes to a concrete UI or event log.
pub trait AgentEventSink {
    /// Publish one runtime event to the host/control plane.
    fn publish(&mut self, event: AgentEvent);
}

/// Runtime plan exposed for user visibility and orchestration tracing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Complete ordered plan entries. Updates replace the whole list.
    pub entries: Vec<PlanEntry>,
}

/// One visible plan step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanEntry {
    /// Human-readable task description.
    pub content: String,
    /// Relative importance of this plan entry.
    pub priority: PlanEntryPriority,
    /// Current lifecycle state.
    pub status: PlanEntryStatus,
}

/// Relative importance of a plan entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanEntryPriority {
    /// Critical path item.
    High,
    /// Important but not blocking.
    Medium,
    /// Nice-to-have or cleanup item.
    Low,
}

/// Current lifecycle state of a plan entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanEntryStatus {
    /// Work has not started.
    Pending,
    /// Work is currently active.
    InProgress,
    /// Work is complete.
    Completed,
}

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

/// Interface implemented by every concrete agent runtime adapter.
///
/// This is the main Stables abstraction over Codex, Claude Code, OpenCode,
/// ACP-compatible agents, or future runtimes. It models the client-to-agent
/// half of ACP without binding callers to JSON-RPC, stdio, or any provider SDK.
pub trait AgentRuntime {
    /// Negotiate contract version and capability sets before sessions are used.
    fn initialize(&mut self, request: InitializeRequest) -> AgentFuture<'_, InitializeResponse>;

    /// Complete runtime authentication when [`InitializeResponse::auth_methods`] is non-empty.
    ///
    /// Default implementation returns [`AgentError::Unsupported`] for runtimes
    /// that do not require or support explicit authentication.
    fn authenticate(
        &mut self,
        _request: AuthenticateRequest,
    ) -> AgentFuture<'_, AuthenticateResponse> {
        unsupported("authenticate")
    }

    /// Create a new interactive session.
    fn create_session(&mut self, request: NewSessionRequest)
    -> AgentFuture<'_, NewSessionResponse>;

    /// Load an existing session with restored history/state.
    ///
    /// Runtimes should only implement this when they advertise
    /// [`AgentCapabilities::load_session`].
    fn load_session(
        &mut self,
        _request: LoadSessionRequest,
    ) -> AgentFuture<'_, LoadSessionResponse> {
        unsupported("load_session")
    }

    /// List sessions known to the runtime.
    ///
    /// Runtimes should only implement this when they advertise
    /// [`SessionCapabilities::list`].
    fn list_sessions(
        &mut self,
        _request: ListSessionsRequest,
    ) -> AgentFuture<'_, ListSessionsResponse> {
        unsupported("list_sessions")
    }

    /// Change the operating mode for an existing session.
    ///
    /// Modes are runtime-defined and should come from [`SessionModeState`].
    fn set_session_mode(
        &mut self,
        _request: SetSessionModeRequest,
    ) -> AgentFuture<'_, SetSessionModeResponse> {
        unsupported("set_session_mode")
    }

    /// Update a runtime-defined session configuration option.
    fn set_session_config(
        &mut self,
        _request: SetSessionConfigRequest,
    ) -> AgentFuture<'_, SetSessionConfigResponse> {
        unsupported("set_session_config")
    }

    /// Process one user prompt turn.
    ///
    /// Implementations should stream intermediate updates through `sink` and
    /// return only when the turn reaches a final [`StopReason`].
    fn prompt(
        &mut self,
        request: PromptRequest,
        sink: &mut dyn AgentEventSink,
    ) -> AgentFuture<'_, PromptResponse>;

    /// Cancel active work for a session.
    ///
    /// A runtime should stop model calls, abort in-flight tools where possible,
    /// flush any final events, and cause the corresponding prompt turn to end
    /// with [`StopReason::Cancelled`].
    fn cancel(&mut self, _request: CancelSessionRequest) -> AgentFuture<'_, ()> {
        unsupported("cancel")
    }
}

/// Host-side callbacks that an agent runtime can request while working.
///
/// This is the Stables abstraction over ACP client methods such as
/// `session/request_permission`, `fs/read_text_file`, and `terminal/create`.
/// The host remains responsible for policy, sandboxing, user approval, and
/// translating results back to the runtime adapter.
pub trait AgentHost {
    /// Advertise which callbacks this host is willing to serve.
    fn capabilities(&self) -> HostCapabilities {
        HostCapabilities::default()
    }

    /// Ask the user or policy engine for permission to run a sensitive tool call.
    fn request_permission(
        &mut self,
        _request: PermissionRequest,
    ) -> AgentFuture<'_, PermissionResponse> {
        unsupported("request_permission")
    }

    /// Read a text file through the host's filesystem policy.
    fn read_text_file(&mut self, _request: ReadTextFileRequest) -> AgentFuture<'_, String> {
        unsupported("read_text_file")
    }

    /// Write a text file through the host's filesystem policy.
    fn write_text_file(&mut self, _request: WriteTextFileRequest) -> AgentFuture<'_, ()> {
        unsupported("write_text_file")
    }

    /// Create a terminal process owned and tracked by the host.
    fn create_terminal(
        &mut self,
        _request: CreateTerminalRequest,
    ) -> AgentFuture<'_, CreateTerminalResponse> {
        unsupported("create_terminal")
    }

    /// Read current output from a host-managed terminal.
    fn terminal_output(
        &mut self,
        _request: TerminalOutputRequest,
    ) -> AgentFuture<'_, TerminalOutputResponse> {
        unsupported("terminal_output")
    }

    /// Release terminal resources once the runtime no longer needs the process.
    fn release_terminal(&mut self, _request: ReleaseTerminalRequest) -> AgentFuture<'_, ()> {
        unsupported("release_terminal")
    }

    /// Kill a terminal process without immediately releasing its buffered output.
    fn kill_terminal(&mut self, _request: KillTerminalRequest) -> AgentFuture<'_, ()> {
        unsupported("kill_terminal")
    }

    /// Wait for a terminal process to exit and return its final status.
    fn wait_for_terminal_exit(
        &mut self,
        _request: WaitForTerminalExitRequest,
    ) -> AgentFuture<'_, WaitForTerminalExitResponse> {
        unsupported("wait_for_terminal_exit")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::Future,
        path::PathBuf,
        pin::pin,
        task::{Context, Poll, Waker},
    };

    fn block_on<F: Future>(future: F) -> F::Output {
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        let mut future = pin!(future);

        loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(output) => return output,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    struct RecordingRuntime {
        events: Vec<AgentEvent>,
    }

    impl AgentRuntime for RecordingRuntime {
        fn initialize(
            &mut self,
            _request: InitializeRequest,
        ) -> AgentFuture<'_, InitializeResponse> {
            Box::pin(async {
                Ok(InitializeResponse {
                    protocol_version: ProtocolVersion::new(1),
                    agent: ImplementationInfo::new("recording", "0.1.0"),
                    capabilities: AgentCapabilities {
                        prompt: PromptCapabilities {
                            image: true,
                            ..PromptCapabilities::default()
                        },
                        ..AgentCapabilities::default()
                    },
                    auth_methods: Vec::new(),
                })
            })
        }

        fn create_session(
            &mut self,
            request: NewSessionRequest,
        ) -> AgentFuture<'_, NewSessionResponse> {
            Box::pin(async move {
                assert_eq!(request.cwd, PathBuf::from("/workspace"));
                Ok(NewSessionResponse {
                    session_id: SessionId::new("session-1"),
                    modes: None,
                    config_options: Vec::new(),
                })
            })
        }

        fn prompt(
            &mut self,
            request: PromptRequest,
            sink: &mut dyn AgentEventSink,
        ) -> AgentFuture<'_, PromptResponse> {
            let event = AgentEvent::AgentMessageChunk {
                session_id: request.session_id,
                content: ContentBlock::text("done"),
            };
            self.events.push(event.clone());
            sink.publish(event);

            Box::pin(async {
                Ok(PromptResponse {
                    stop_reason: StopReason::EndTurn,
                })
            })
        }
    }

    #[derive(Default)]
    struct RecordingSink {
        events: Vec<AgentEvent>,
    }

    impl AgentEventSink for RecordingSink {
        fn publish(&mut self, event: AgentEvent) {
            self.events.push(event);
        }
    }

    struct HostHarness;

    impl AgentHost for HostHarness {
        fn capabilities(&self) -> HostCapabilities {
            HostCapabilities {
                filesystem: FileSystemCapabilities {
                    read_text_file: true,
                    write_text_file: true,
                },
                terminal: true,
                permissions: true,
            }
        }

        fn request_permission(
            &mut self,
            request: PermissionRequest,
        ) -> AgentFuture<'_, PermissionResponse> {
            Box::pin(async move {
                assert_eq!(request.session_id, SessionId::new("session-1"));
                Ok(PermissionResponse {
                    outcome: PermissionOutcome::Selected {
                        option_id: PermissionOptionId::new("allow_once"),
                    },
                })
            })
        }

        fn read_text_file(&mut self, request: ReadTextFileRequest) -> AgentFuture<'_, String> {
            Box::pin(async move {
                assert_eq!(request.path, PathBuf::from("/workspace/README.md"));
                Ok("# Stables".to_string())
            })
        }
    }

    #[test]
    fn runtime_trait_maps_acp_lifecycle_and_streaming_updates() {
        let mut runtime: Box<dyn AgentRuntime + Send> =
            Box::new(RecordingRuntime { events: Vec::new() });

        let initialized = block_on(runtime.initialize(InitializeRequest {
            protocol_version: ProtocolVersion::new(1),
            host: ImplementationInfo::new("stables", "0.1.0"),
            host_capabilities: HostCapabilities::default(),
        }))
        .unwrap();

        assert_eq!(initialized.agent.name, "recording");
        assert!(initialized.capabilities.prompt.image);

        let session = block_on(runtime.create_session(NewSessionRequest {
            cwd: PathBuf::from("/workspace"),
            mcp_servers: Vec::new(),
        }))
        .unwrap();

        let mut sink = RecordingSink::default();
        let response = block_on(runtime.prompt(
            PromptRequest {
                session_id: session.session_id.clone(),
                prompt: vec![ContentBlock::text("build the agent module")],
            },
            &mut sink,
        ))
        .unwrap();

        assert_eq!(response.stop_reason, StopReason::EndTurn);
        assert_eq!(
            sink.events,
            vec![AgentEvent::AgentMessageChunk {
                session_id: SessionId::new("session-1"),
                content: ContentBlock::text("done"),
            }]
        );
    }

    #[test]
    fn host_trait_maps_acp_client_capabilities() {
        let mut host = HostHarness;

        let capabilities = host.capabilities();
        assert!(capabilities.filesystem.read_text_file);
        assert!(capabilities.filesystem.write_text_file);
        assert!(capabilities.terminal);

        let permission = block_on(host.request_permission(PermissionRequest {
            session_id: SessionId::new("session-1"),
            tool_call: ToolCallUpdate {
                id: ToolCallId::new("tool-1"),
                title: Some("Edit file".to_string()),
                status: Some(ToolCallStatus::Pending),
                content: Vec::new(),
            },
            options: vec![PermissionOption {
                id: PermissionOptionId::new("allow_once"),
                name: "Allow once".to_string(),
                kind: PermissionOptionKind::AllowOnce,
            }],
        }))
        .unwrap();

        assert_eq!(
            permission.outcome,
            PermissionOutcome::Selected {
                option_id: PermissionOptionId::new("allow_once"),
            }
        );

        let contents = block_on(host.read_text_file(ReadTextFileRequest {
            session_id: SessionId::new("session-1"),
            path: PathBuf::from("/workspace/README.md"),
            line: None,
            limit: None,
        }))
        .unwrap();

        assert_eq!(contents, "# Stables");
    }
}
