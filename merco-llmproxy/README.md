# merco-llmproxy

A Rust library providing a unified interface for various Large Language Model (LLM) providers, inspired by [LiteLLM](https://github.com/BerriAI/litellm).

This crate aims to simplify interaction with different LLMs by offering:

*   A common configuration structure (`LlmConfig`).
*   A unified asynchronous trait (`LlmProvider`) for chat completions.
*   Support for multiple providers (currently OpenAI-compatible APIs and Ollama).
*   Basic support for non-streaming tool calls (function calling).

## Current Status

*   **Providers:** OpenAI (including proxies like OpenRouter), Ollama.
*   **Features:** Non-streaming Chat Completion, Non-streaming Tool Calls.
*   **Limitations:** Streaming Tool Calls are currently **not** supported reliably due to SSE parsing complexities and Ollama's JSON mode limitations.

## Installation

Add this crate to your `Cargo.toml` dependencies:

```toml
[dependencies]
merco-llmproxy = { git = "<your-repo-url>" } # Or path = "..." if local
# Required peer dependencies (ensure versions are compatible)
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1"
futures = "0.3"
thiserror = "1.0"
```

*(Replace `<your-repo-url>` with the actual repository URL once published.)*

## Usage

### 1. Configuration

Create an `LlmConfig` specifying the provider, model, and any necessary credentials or URLs.

```rust
use merco_llmproxy::{LlmConfig, Provider};

// Example: Configure for a local Ollama model
let ollama_config = LlmConfig::new(Provider::Ollama, "qwen3:4b".to_string());

// Example: Configure for OpenAI via OpenRouter (requires API key)
let openrouter_api_key = std::env::var("OPENROUTER_API_KEY")
    .expect("OPENROUTER_API_KEY must be set");

let openrouter_config = LlmConfig::new(
        Provider::OpenAI, // Use OpenAI provider type for compatible APIs
        "mistralai/mistral-7b-instruct-v0.1".to_string() // Specify the OpenRouter model ID
    )
    .with_base_url("https://openrouter.ai/api/v1".to_string())
    .with_api_key(openrouter_api_key);
```

**Environment Variables:**

*   For providers requiring API keys (like OpenAI/OpenRouter), ensure the corresponding key is set (e.g., `OPENROUTER_API_KEY`).

### 2. Get Provider Instance

Use the `get_provider` function to obtain a trait object (`Arc<dyn LlmProvider>`) based on the configuration.

```rust
# use merco_llmproxy::{LlmConfig, Provider, get_provider};
# let config = LlmConfig::new(Provider::Ollama, "qwen3:4b".to_string());
let provider = match get_provider(config) {
    Ok(p) => p,
    Err(e) => {
        eprintln!("Failed to get provider: {}", e);
        // Handle error appropriately
        panic!(); 
    }
};
```

### 3. Simple Chat Completion (Non-streaming)

Create a `CompletionRequest` and call the `completion` method.

```rust
# use merco_llmproxy::{LlmConfig, Provider, get_provider};
# use merco_llmproxy::traits::{ChatMessage, CompletionRequest, CompletionKind};
# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
# let config = LlmConfig::new(Provider::Ollama, "qwen3:4b".to_string());
# let provider = get_provider(config)?;
let request = CompletionRequest {
    model: "qwen3:4b".to_string(), // Or the model configured in LlmConfig
    messages: vec![
        ChatMessage {
            role: "system".to_string(),
            content: Some("You are a helpful assistant.".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
        ChatMessage {
            role: "user".to_string(),
            content: Some("Why is the sky blue?".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ],
    temperature: Some(0.7),
    max_tokens: Some(100),
    tools: None, // No tools needed for this request
};

match provider.completion(request).await {
    Ok(response) => {
        println!("Finish Reason: {:?}", response.finish_reason);
        if let Some(usage) = response.usage {
            println!("Usage: Prompt={}, Completion={}, Total={}", 
                usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
        }
        match response.kind {
            CompletionKind::Message { content } => {
                println!("Response Content:\n{}", content);
            }
            CompletionKind::ToolCall { tool_calls } => {
                println!("Unexpected tool call requested: {:?}", tool_calls);
            }
        }
    }
    Err(e) => {
        eprintln!("Completion Error: {}", e);
    }
}
# Ok(())
# }
```

### 4. Tool Calling (Non-streaming)

Define tools, include them in the `CompletionRequest`, and handle the `CompletionKind::ToolCall` variant in the response.

```rust
use merco_llmproxy::{
    LlmConfig, Provider, get_provider,
    traits::{*
        ChatMessage, CompletionKind, CompletionRequest, JsonSchema, Tool,
        ToolCallFunction, ToolCallRequest, TokenUsage,
    }
};
use serde::Deserialize;
use serde_json::json;

// 1. Define your tool implementation and argument struct
#[derive(Deserialize)]
struct SumArgs { a: i64, b: i64 }
fn sum_numbers(a: i64, b: i64) -> i64 { a + b }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 2. Define the tool structure for the LLM
    let sum_tool = Tool {
        name: "sum_numbers".to_string(),
        description: "Calculates the sum of two integers.".to_string(),
        parameters: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some({
                let mut props = serde_json::Map::new();
                props.insert("a".to_string(), json!({ "type": "integer" }));
                props.insert("b".to_string(), json!({ "type": "integer" }));
                props
            }),
            required: Some(vec!["a".to_string(), "b".to_string()]),
        },
    };

    // 3. Configure Provider (e.g., OpenRouter - requires env var)
    let api_key = std::env::var("OPENROUTER_API_KEY")?;
    let config = LlmConfig::new(Provider::OpenAI, "mistralai/mistral-7b-instruct-v0.1".to_string())
        .with_base_url("https://openrouter.ai/api/v1".to_string())
        .with_api_key(api_key);
    let provider = get_provider(config)?;

    // 4. Create Request with Tools
    let request = CompletionRequest {
        model: "mistralai/mistral-7b-instruct-v0.1".to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: Some("What is the sum of 123 and 456?".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        temperature: Some(0.1),
        max_tokens: Some(150),
        tools: Some(vec![sum_tool]), // Provide the tool
    };

    // 5. Make Request and Handle Response
    match provider.completion(request).await {
        Ok(response) => {
            println!("Finish Reason: {:?}", response.finish_reason);
            match response.kind {
                CompletionKind::Message { content } => {
                    println!("Response Content:\n{}", content);
                }
                CompletionKind::ToolCall { tool_calls } => {
                    println!("Tool Calls Requested:");
                    for call in tool_calls {
                        println!("  ID: {}, Function: {}", call.id, call.function.name);
                        if call.function.name == "sum_numbers" {
                            match serde_json::from_str::<SumArgs>(&call.function.arguments) {
                                Ok(args) => {
                                    let result = sum_numbers(args.a, args.b);
                                    println!("  -> Simulated Result: {}", result);
                                    // Next step: Send result back in a new request
                                    // with role="tool", tool_call_id=call.id, content=result.to_string()
                                }
                                Err(e) => eprintln!("   -> Arg parse error: {}", e),
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Completion Error: {}", e);
        }
    }
    Ok(())
}
```

## Running Examples

The code in `src/main.rs` contains example usage similar to the snippets above. You can run it using:

```bash
# For OpenAI/OpenRouter tests:
export OPENROUTER_API_KEY="your-key-here"

# For Ollama tests, ensure Ollama server is running with the model:
# ollama pull qwen3:4b 

cargo run
```

## Contributing

Contributions are welcome! Please feel free to open issues or pull requests.

## License
