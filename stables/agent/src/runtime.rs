use crate::{
    capabilities::{AgentCapabilities, HostCapabilities},
    error::{AgentFuture, unsupported},
    event::AgentEventSink,
    identity::{AuthMethod, AuthMethodId, ImplementationInfo, ProtocolVersion},
    permission::{PermissionRequest, PermissionResponse},
    session::{
        CancelSessionRequest, ListSessionsRequest, ListSessionsResponse, LoadSessionRequest,
        LoadSessionResponse, NewSessionRequest, NewSessionResponse, PromptRequest, PromptResponse,
        SetSessionConfigRequest, SetSessionConfigResponse, SetSessionModeRequest,
        SetSessionModeResponse,
    },
    terminal::{
        CreateTerminalRequest, CreateTerminalResponse, KillTerminalRequest, ReadTextFileRequest,
        ReleaseTerminalRequest, TerminalOutputRequest, TerminalOutputResponse,
        WaitForTerminalExitRequest, WaitForTerminalExitResponse, WriteTextFileRequest,
    },
};

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

/// Request to complete one of the auth methods returned during initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticateRequest {
    /// Auth method selected by the host/user.
    pub method_id: AuthMethodId,
}

/// Empty response indicating authentication completed successfully.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthenticateResponse;

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
    /// Default implementation returns `AgentError::Unsupported` for runtimes
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
    /// `SessionCapabilities::list`.
    fn list_sessions(
        &mut self,
        _request: ListSessionsRequest,
    ) -> AgentFuture<'_, ListSessionsResponse> {
        unsupported("list_sessions")
    }

    /// Change the operating mode for an existing session.
    ///
    /// Modes are runtime-defined and should come from `SessionModeState`.
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
    /// return only when the turn reaches a final `StopReason`.
    fn prompt(
        &mut self,
        request: PromptRequest,
        sink: &mut dyn AgentEventSink,
    ) -> AgentFuture<'_, PromptResponse>;

    /// Cancel active work for a session.
    ///
    /// A runtime should stop model calls, abort in-flight tools where possible,
    /// flush any final events, and cause the corresponding prompt turn to end
    /// with `StopReason::Cancelled`.
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
