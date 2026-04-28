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
