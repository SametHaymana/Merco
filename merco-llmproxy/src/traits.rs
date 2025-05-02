use async_trait::async_trait;
use futures::stream::Stream; // Requires the `futures` crate
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue; // For JSON Schema representation
use std::pin::Pin;
use thiserror::Error;

// --- Tool Calling Structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: JsonSchema,
}

// Represents a JSON Schema definition
// Using serde_json::Value for flexibility, could be more strongly typed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    #[serde(rename = "type")]
    pub schema_type: String, // Typically "object"
    pub properties: Option<serde_json::Map<String, JsonValue>>,
    pub required: Option<Vec<String>>,
    // Add other JSON schema fields if needed (e.g., description for properties)
}

// --- Request/Response Structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    // TODO: Add tool_choice option (e.g., "auto", "required", {"type": "function", "function": {"name": "..."}})
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>, // Content can be None for tool calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>, // Used for responses *from* a tool
}

// Represents a tool call requested by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub id: String, // Unique ID for this specific call
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String, // Arguments are often a JSON string
}

// Represents the different kinds of completion results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)] // Allows deserializing into either Message or ToolCall variant
pub enum CompletionKind {
    Message { content: String },
    ToolCall { tool_calls: Vec<ToolCallRequest> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    // Replaced direct content with CompletionKind
    #[serde(flatten)] // Embed the CompletionKind fields directly
    pub kind: CompletionKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

// Represents the different kinds of stream chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamContentDelta {
    #[serde(rename = "content")]
    Text(String),
    #[serde(rename = "tool_calls")]
    ToolCallDelta(Vec<ToolCallStreamDelta>), // Tool call parts arrive incrementally
}

// Represents an incremental part of a tool call in a stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallStreamDelta {
    pub index: usize, // The index of the tool call this delta applies to
    pub id: Option<String>, // ID usually appears in the first delta for a call
    pub function: Option<ToolCallFunctionStreamDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunctionStreamDelta {
    pub name: Option<String>, // Name usually appears in the first delta
    pub arguments: Option<String>, // Argument JSON string chunks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionStreamChunk {
    // Replaced 'delta: String' with 'delta: StreamContentDelta'
    pub delta: StreamContentDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("API request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("API response error: {status}: {message}")]
    ApiError { status: u16, message: String },
    #[error("Failed to parse API response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Configuration error: {0}")]
    ConfigError(String), // Reuse or map from config::ConfigError
    #[error("Stream failed: {0}")]
    StreamError(String),
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
    #[error("Tool use response format error: {0}")]
    ToolFormatError(String),
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    #[error("An unexpected error occurred: {0}")]
    Unexpected(String),
}

// Define a type alias for the stream
pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<CompletionStreamChunk, ProviderError>> + Send>>;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generates a non-streaming completion.
    async fn completion(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError>;

    /// Generates a streaming completion.
    async fn completion_stream(&self, request: CompletionRequest) -> Result<CompletionStream, ProviderError>;

    // Optional: Add a method to get provider-specific information or capabilities
    // fn capabilities(&self) -> ProviderCapabilities;
} 