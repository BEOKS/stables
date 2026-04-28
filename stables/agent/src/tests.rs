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
    fn initialize(&mut self, _request: InitializeRequest) -> AgentFuture<'_, InitializeResponse> {
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
