use crate::{
    content::ContentBlock,
    identity::SessionId,
    plan::Plan,
    session::{SessionConfigOption, SessionModeId},
    tool::{AvailableCommand, ToolCall, ToolCallUpdate},
};

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
