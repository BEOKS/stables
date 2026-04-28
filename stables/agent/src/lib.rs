//! Agent runtime contracts for Stables.
//!
//! This crate defines Stables' internal agent domain model. It is intentionally
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

mod capabilities;
mod content;
mod error;
mod event;
mod identity;
mod permission;
mod plan;
mod runtime;
mod session;
mod terminal;
#[cfg(test)]
mod tests;
mod tool;

pub use capabilities::*;
pub use content::*;
pub use error::*;
pub use event::*;
pub use identity::*;
pub use permission::*;
pub use plan::*;
pub use runtime::*;
pub use session::*;
pub use terminal::*;
pub use tool::*;
