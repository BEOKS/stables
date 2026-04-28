/// Feature set implemented by the agent runtime.
///
/// This is the agent side of capability negotiation. It tells Stables which
/// optional runtime operations are legal to call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentCapabilities {
    /// Whether the runtime can restore an existing session.
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

/// File-system operations available through host callbacks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileSystemCapabilities {
    /// Runtime may ask the host to read text files.
    pub read_text_file: bool,
    /// Runtime may ask the host to write text files.
    pub write_text_file: bool,
}
