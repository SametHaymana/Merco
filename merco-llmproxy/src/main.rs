// This file is usually empty for libraries
// If you want an executable, you can add main function here
// or create a separate binary crate in the project.

use merco_llmproxy::config::{LlmConfig, Provider};
use merco_llmproxy::traits::{
    ChatMessage, CompletionKind, CompletionRequest, JsonSchema, Tool, ToolCallFunction,
    ToolCallRequest, // Keep structs needed for tool definition/handling
    // Remove structs only used by streaming test or unused now:
    // LlmProvider, ProviderError, TokenUsage, CompletionResponse, CompletionStreamChunk, StreamContentDelta, ToolCallStreamDelta 
};
use merco_llmproxy::get_provider;
use serde_json::{self, json};
use serde::Deserialize;
// Removed unused imports: HashMap, env

// --- Tool Implementation (Example) ---
#[allow(dead_code)] // Allow dead code since only used in test/example
fn sum_numbers(a: i64, b: i64) -> i64 {
    a + b
}

#[derive(Deserialize)]
#[allow(dead_code)] // Allow dead code since only used in test/example
struct SumArgs {
    a: i64,
    b: i64,
}

// --- Main Test Logic ---

#[tokio::main]
async fn main() {
    // Test OpenAI Tool Call via OpenRouter
    test_openai_tools().await;
    println!("\n-----------------------------\n");
    // Test Ollama Tool Call (Expecting Success)
    test_ollama_tools().await;
}

/// Tests OpenAI provider non-streaming tool call via OpenRouter
async fn test_openai_tools() {
    println!("--- Testing OpenAI Provider Tool Call (via OpenRouter) ---");

     let api_key = match std::env::var("OPENROUTER_API_KEY") { // Use std::env directly
        Ok(key) => key,
        Err(_) => {
            eprintln!("Skipping OpenAI test: OPENROUTER_API_KEY environment variable not set.");
            return;
        }
    };

    let sum_tool = create_sum_tool();
    let model_name = "mistralai/mistral-7b-instruct-v0.1".to_string(); 
    let config = LlmConfig::new(Provider::OpenAI, model_name.clone())
        .with_base_url("https://openrouter.ai/api/v1".to_string())
        .with_api_key(api_key);

     let provider = match get_provider(config) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to get OpenAI provider: {}", e);
            return;
        }
    };

    let request = create_tool_request(model_name, sum_tool);

    println!("\n--- Testing Non-Streaming Tool Call ---");
    match provider.completion(request).await { // Removed clone as streaming is gone
        Ok(response) => {
            handle_completion_response(response);
        }
        Err(e) => {
            eprintln!("Completion Error: {}", e);
        }
    }
}

/// Tests Ollama provider non-streaming tool call
async fn test_ollama_tools() {
    println!("--- Testing Ollama Provider Tool Call ---");

    let sum_tool = create_sum_tool();
    let model_name = "qwen3:4b".to_string();
    let config = LlmConfig::new(Provider::Ollama, model_name.clone());

    let provider = match get_provider(config) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to get Ollama provider: {}", e);
            return;
        }
    };

    let request = create_tool_request(model_name, sum_tool);

    println!("\n--- Testing Non-Streaming Tool Call ---");
    match provider.completion(request).await {
         Ok(response) => {
            handle_completion_response(response);
        }
        Err(e) => {
            eprintln!("Completion Error: {}", e);
        }
    }
}

/// Helper to create the sum tool definition
fn create_sum_tool() -> Tool {
     Tool {
        name: "sum_numbers".to_string(),
        description: "Calculates the sum of two integers.".to_string(),
        parameters: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some({
                let mut props = serde_json::Map::new();
                props.insert(
                    "a".to_string(),
                    json!({ "type": "integer", "description": "First integer" }),
                );
                props.insert(
                    "b".to_string(),
                    json!({ "type": "integer", "description": "Second integer" }),
                );
                props
            }),
            required: Some(vec!["a".to_string(), "b".to_string()]),
        },
    }
}

/// Helper to create a completion request asking for a sum
fn create_tool_request(model_name: String, tool: Tool) -> CompletionRequest {
     CompletionRequest {
        model: model_name,
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
        tools: Some(vec![tool]),
    }
}

/// Helper to handle and print the completion response (message or tool call)
fn handle_completion_response(response: merco_llmproxy::traits::CompletionResponse) { 
    // Need to import CompletionResponse for type hint
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
            println!("Tool Calls Requested: {:?}", tool_calls);
            for call in tool_calls {
                if call.function.name == "sum_numbers" {
                     match serde_json::from_str::<SumArgs>(&call.function.arguments) {
                         Ok(args) => {
                             let result = sum_numbers(args.a, args.b);
                             println!("  -> Simulated Result: {}", result);
                         }
                         Err(e) => {
                             eprintln!("  -> Failed to parse args for {}: {}", call.id, e);
                         }
                     }
                 }
            }
        }
    }
}

/* --- Streaming Tool Call Test (Commented Out) ---
// ... remains commented out ...
*/ 