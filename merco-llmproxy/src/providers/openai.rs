use crate::config::{LlmConfig, Provider};
use crate::traits::{
    ChatMessage, CompletionKind, CompletionRequest, CompletionResponse, CompletionStream,
    CompletionStreamChunk, JsonSchema, LlmProvider, ProviderError, StreamContentDelta, Tool,
    ToolCallFunction, ToolCallFunctionStreamDelta, ToolCallRequest, ToolCallStreamDelta, TokenUsage,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::{Stream, StreamExt, TryStreamExt};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{self, json, Value as JsonValue};
use std::collections::HashMap; // Needed for assembling stream tool calls
use std::sync::{Arc, Mutex}; // Added Arc, Mutex
use std::time::Duration;
use serde::de::Error as DeError;
use std::pin::Pin;

const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 120;

// --- OpenAI Specific API Structures ---

// Map our generic Tool struct to OpenAI's format
#[derive(Serialize, Debug)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String, // Always "function" for now
    function: OpenAIFunctionDef,
}

#[derive(Serialize, Debug)]
struct OpenAIFunctionDef {
    name: String,
    description: String,
    parameters: JsonSchema, // Re-use our JsonSchema struct directly
}

#[derive(Serialize, Debug)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<JsonValue>, // Can be "auto", "none", or specific tool spec
}

#[derive(Deserialize, Debug)]
struct OpenAIChatResponse {
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Deserialize, Debug)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

// OpenAI's message can contain text content OR tool calls
#[derive(Deserialize, Debug, Clone)]
struct OpenAIMessage {
    role: String,
    content: Option<String>, // Can be null when tool_calls are present
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

// Represents a tool call requested by OpenAI
#[derive(Deserialize, Debug, Clone)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String, // Should be "function"
    function: OpenAIFunctionCall,
}

#[derive(Deserialize, Debug, Clone)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String, // JSON string arguments
}

#[derive(Deserialize, Debug, Clone, Copy)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// --- Streaming Structures ---

#[derive(Deserialize, Debug)]
struct OpenAIChatStreamResponse {
    model: String,
    choices: Vec<OpenAIStreamChoice>,
    usage: Option<OpenAIUsage>, // Only in final chunk from some models/APIs
}

#[derive(Deserialize, Debug)]
struct OpenAIStreamChoice {
    index: u32,
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

// Delta can contain text OR tool call parts
#[derive(Deserialize, Debug)]
struct OpenAIStreamDelta {
    role: Option<String>, // Usually only present in the first chunk
    content: Option<String>, // Text delta
    tool_calls: Option<Vec<OpenAIStreamToolCallDelta>>, // Tool call delta
}

// Represents an incremental part of a tool call in the stream
#[derive(Deserialize, Debug)]
struct OpenAIStreamToolCallDelta {
    index: usize, // Index of the tool call this delta is for
    id: Option<String>, // ID usually appears once per tool call
    #[serde(rename = "type")]
    tool_type: Option<String>, // Usually "function"
    function: Option<OpenAIStreamFunctionDelta>,
}

#[derive(Deserialize, Debug)]
struct OpenAIStreamFunctionDelta {
    name: Option<String>, // Name usually appears once per tool call
    arguments: Option<String>, // Argument JSON string chunks
}

// --- Provider Implementation ---

#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    config: LlmConfig,
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(config: LlmConfig) -> Self {
        let api_key = config
            .api_key
            .clone()
            .expect("OpenAI provider requires an API key");

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| OPENAI_BASE_URL.to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("Failed to build Reqwest client");

        Self { config, client, api_key, base_url }
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .expect("Failed to create auth header"),
        );
        // Add headers required by specific proxies like OpenRouter if necessary
        // headers.insert("HTTP-Referer", HeaderValue::from_static("YOUR_SITE_URL"));
        // headers.insert("X-Title", HeaderValue::from_static("YOUR_APP_NAME"));
        headers
    }

    // Helper to map generic Tools to OpenAI Tools
    fn map_tools_to_openai(tools: Option<&Vec<Tool>>) -> Option<Vec<OpenAITool>> {
        tools.map(|ts| {
            ts.iter()
                .map(|tool| OpenAITool {
                    tool_type: "function".to_string(),
                    function: OpenAIFunctionDef {
                        name: tool.name.clone(),
                        description: tool.description.clone(),
                        parameters: tool.parameters.clone(),
                    },
                })
                .collect()
        })
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn completion(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError> {
        if self.config.provider != Provider::OpenAI {
            return Err(ProviderError::ConfigError(
                "Invalid provider configured for OpenAIProvider".to_string(),
            ));
        }

        let openai_request = OpenAIChatRequest {
            model: request.model.clone(),
            messages: request.messages.clone(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false,
            tools: Self::map_tools_to_openai(request.tools.as_ref()),
            tool_choice: request.tools.as_ref().map(|_| json!("auto")), // Default to auto if tools are provided
        };

        let url = format!("{}/chat/completions", self.base_url);
        let headers = self.build_headers();

        let res = self
            .client
            .post(&url)
            .headers(headers)
            .json(&openai_request)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status().as_u16();
            let error_body = res.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(ProviderError::ApiError { status, message: error_body });
        }

        let openai_response: OpenAIChatResponse = res.json().await?;

        let first_choice = openai_response.choices.into_iter().next()
            .ok_or_else(|| ProviderError::ParseError(serde_json::Error::custom("No choices found in OpenAI response")))?;

        let usage = openai_response.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        // Check if the response contains tool calls or a message
        let kind = if let Some(tool_calls) = first_choice.message.tool_calls {
            // Map OpenAI tool calls to our generic format
            let generic_tool_calls = tool_calls
                .into_iter()
                .map(|tc| ToolCallRequest {
                    id: tc.id,
                    function: ToolCallFunction {
                        name: tc.function.name,
                        arguments: tc.function.arguments,
                    },
                })
                .collect();
            CompletionKind::ToolCall { tool_calls: generic_tool_calls }
        } else if let Some(content) = first_choice.message.content {
            CompletionKind::Message { content }
        } else {
            // Should not happen if finish_reason is not tool_calls
             if first_choice.finish_reason == Some("tool_calls".to_string()) {
                 // It's possible to get finish_reason tool_calls but empty message content/tool_calls in rare cases?
                 // Return empty tool call list for now
                 CompletionKind::ToolCall { tool_calls: vec![] }
             } else {
                 // Or maybe it finished normally but content was empty/null?
                 CompletionKind::Message { content: "".to_string() }
             }
        };

        Ok(CompletionResponse {
            kind,
            usage,
            finish_reason: first_choice.finish_reason,
        })
    }

    async fn completion_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionStream, ProviderError> {
        // --- TEMPORARY: Disable streaming tool calls due to parsing issues ---
        if request.tools.is_some() {
            return Err(ProviderError::Unsupported(
                "Streaming tool calls are not currently supported by the OpenAI provider implementation.".to_string()
            ));
        }
        // --- END TEMPORARY --- 

        if self.config.provider != Provider::OpenAI {
            return Err(ProviderError::ConfigError(
                "Invalid provider configured for OpenAIProvider".to_string(),
            ));
        }

        let openai_request = OpenAIChatRequest {
            model: request.model.clone(),
            messages: request.messages.clone(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: true,
            tools: Self::map_tools_to_openai(request.tools.as_ref()),
            tool_choice: request.tools.as_ref().map(|_| json!("auto")), // Default to auto
        };

        let url = format!("{}/chat/completions", self.base_url);
        let headers = self.build_headers();

        let res = self
            .client
            .post(&url)
            .headers(headers)
            .json(&openai_request)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status().as_u16();
            let error_body = res.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(ProviderError::ApiError { status, message: error_body });
        }

        let sse_stream = res.bytes_stream().map_err(ProviderError::RequestError);

        // State wrapped in Arc<Mutex> for shared mutable access
        let tool_call_aggregator = Arc::new(Mutex::new(HashMap::<usize, ToolCallStreamDelta>::new()));

        let chunk_stream = sse_stream.try_filter_map(move |chunk: Bytes| {
            // Clone the Arc for the async block, this is cheap
            let state_lock = Arc::clone(&tool_call_aggregator);

            async move {
                let lines = chunk.split(|&b| b == b'\n');
                let mut result_chunk: Option<CompletionStreamChunk> = None;
                let mut final_usage: Option<OpenAIUsage> = None;
                let mut final_reason: Option<String> = None;

                // Lock mutex for the duration needed to process this chunk
                let mut current_tool_calls = state_lock.lock().map_err(|_| {
                    ProviderError::Unexpected("Mutex poisoned in stream processing".to_string())
                })?;

                for line in lines {
                    if line.starts_with(b"data: ") {
                        let data = &line[6..];
                        if data.is_empty() || data == b"[DONE]" {
                            continue;
                        }

                        match serde_json::from_slice::<OpenAIChatStreamResponse>(data) {
                            Ok(openai_chunk) => {
                                if let Some(usage) = openai_chunk.usage {
                                    final_usage = Some(usage);
                                }

                                if let Some(choice) = openai_chunk.choices.into_iter().next() {
                                     if let Some(reason) = choice.finish_reason {
                                         final_reason = Some(reason);
                                     }

                                    if let Some(text_delta) = choice.delta.content {
                                        if !text_delta.is_empty() {
                                            result_chunk = Some(CompletionStreamChunk {
                                                delta: StreamContentDelta::Text(text_delta),
                                                usage: None,
                                                finish_reason: None,
                                            });
                                            // Clear the shared state when text is received
                                            current_tool_calls.clear();
                                        }
                                    } else if let Some(tool_deltas) = choice.delta.tool_calls {
                                        let mut generic_deltas = Vec::new();
                                        for tool_delta in tool_deltas {
                                            // Access and modify the state behind the mutex lock
                                            let entry = current_tool_calls
                                                .entry(tool_delta.index)
                                                .or_insert_with(|| ToolCallStreamDelta {
                                                    index: tool_delta.index,
                                                    id: None,
                                                    function: None,
                                                });

                                            if let Some(id) = tool_delta.id { entry.id = Some(id); }
                                            if let Some(func_delta) = tool_delta.function {
                                                let func_entry = entry.function.get_or_insert_with(|| {
                                                    ToolCallFunctionStreamDelta {
                                                        name: None,
                                                        arguments: None,
                                                    }
                                                });
                                                if let Some(name) = func_delta.name { func_entry.name = Some(name); }
                                                if let Some(args_chunk) = func_delta.arguments { 
                                                     // DEBUG: Print incoming arg chunk
                                                     eprintln!("--> DEBUG: Received args_chunk: {:?}", args_chunk);
                                                     let current_args = func_entry.arguments.clone().unwrap_or_default();
                                                     // DEBUG: Print state *before* appending
                                                     eprintln!("--> DEBUG: current_args: {:?}", current_args);
                                                     func_entry.arguments = Some(current_args + &args_chunk);
                                                     // DEBUG: Print state *after* appending
                                                     eprintln!("--> DEBUG: func_entry.arguments after: {:?}", func_entry.arguments);
                                                 }
                                            }
                                            generic_deltas.push(entry.clone());
                                        }
                                        if !generic_deltas.is_empty() {
                                            result_chunk = Some(CompletionStreamChunk {
                                                delta: StreamContentDelta::ToolCallDelta(generic_deltas),
                                                usage: None,
                                                finish_reason: None,
                                            });
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse OpenAI SSE chunk: {:?}, data: {}", e, String::from_utf8_lossy(data));
                                return Err(ProviderError::ParseError(e));
                            }
                        }
                    }
                }
                // Mutex guard `current_tool_calls` is dropped here, unlocking the mutex

                // If final info collected, create final chunk (unless we already generated a chunk)
                if result_chunk.is_none() && (final_reason.is_some() || final_usage.is_some()) {
                     result_chunk = Some(CompletionStreamChunk {
                         delta: StreamContentDelta::Text("".to_string()),
                         usage: final_usage.map(|u| TokenUsage {
                                 prompt_tokens: u.prompt_tokens,
                                 completion_tokens: u.completion_tokens,
                                 total_tokens: u.total_tokens,
                             }),
                         finish_reason: final_reason,
                     });
                 }

                 Ok(result_chunk)
            }
        });

        Ok(Box::pin(chunk_stream))
    }
} 