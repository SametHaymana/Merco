use merco_agents::agent::agent::Agent;
use merco_agents::task::task::{Task, JsonFieldType, JsonField};
use merco_llmproxy::{LlmConfig, Provider, get_tools_by_names, merco_tool};
use merco_agents::agent::agent::AgentLLMConfig;

use dotenv::dotenv;

#[merco_tool(description = "A tool to get the current time")]
pub fn get_current_time() -> String {
    println!("get_current_time tool called");
    
    // Get the current system time
    let now = std::time::SystemTime::now();
    // Convert SystemTime to DateTime<Local> using the chrono crate
    let datetime: chrono::DateTime<chrono::Local> = now.into();
    // Format the datetime into a human-readable string
    datetime.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");

    let llm_config = LlmConfig::new(Provider::OpenAI)
        .with_base_url("https://openrouter.ai/api/v1".to_string())
        .with_api_key(api_key);


    let model: &str = "openai/gpt-4.1";
    let agent_llm_config = AgentLLMConfig::new(llm_config, model.to_string(), 0.0, 1000);

    // Test without tools first to verify JSON validation works
    let agent_no_tools = Agent::new(
        agent_llm_config.clone(),
        "You are a helpful assistant that provides information in structured formats.".to_string(),
        vec!["Provide accurate responses in the requested format.".to_string()],
        vec![], // No tools
    );

    // Example 1: Simple text task (no validation)
    println!("=== Example 1: Text Output Task (No Tools) ===");
    let text_task = Task::new(
        "What is 2 + 2?".to_string(),
        Some("A simple mathematical answer.".to_string()),
    );

    match agent_no_tools.call(text_task).await {
        Ok(result) => println!("Text Task Result: {}", result),
        Err(e) => println!("Text Task Error: {}", e),
    }

    // Example 2: JSON task with validation (no tools)
    println!("\n=== Example 2: JSON Output Task with Validation (No Tools) ===");
    let json_task = Task::new_simple_json(
        "Calculate 5 + 7 and provide the result in structured format.".to_string(),
        Some("Mathematical calculation result in JSON format.".to_string()),
        vec![
            ("question".to_string(), JsonFieldType::String),
            ("answer".to_string(), JsonFieldType::Number),
            ("operation".to_string(), JsonFieldType::String),
        ],
        true, // strict mode
    );

    match agent_no_tools.call(json_task).await {
        Ok(result) => {
            println!("JSON Task Result: {}", result);
            // Parse and pretty-print the JSON
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("Parsed JSON: {}", serde_json::to_string_pretty(&parsed)?);
            }
        },
        Err(e) => println!("JSON Task Error: {}", e),
    }

    // Example 3: Complex JSON with nested objects
    println!("\n=== Example 3: Nested Object JSON Validation ===");
    let nested_task = Task::new_with_json_output(
        "Create a user profile for a fictional character named Alex who is 25 years old and lives in San Francisco.".to_string(),
        Some("User profile with personal and address information in nested JSON format.".to_string()),
        vec![
            // Required fields
            JsonField {
                name: "user_id".to_string(),
                field_type: JsonFieldType::Number,
                description: Some("Unique user identifier".to_string()),
            },
            JsonField {
                name: "personal_info".to_string(),
                field_type: JsonFieldType::Object,
                description: Some("Personal information object containing name, age, email".to_string()),
            },
            JsonField {
                name: "address".to_string(),
                field_type: JsonFieldType::Object,
                description: Some("Address object containing city, state, country".to_string()),
            },
            JsonField {
                name: "active".to_string(),
                field_type: JsonFieldType::Boolean,
                description: Some("Whether the user account is active".to_string()),
            },
        ],
        vec![
            // Optional fields
            JsonField {
                name: "preferences".to_string(),
                field_type: JsonFieldType::Object,
                description: Some("User preferences object".to_string()),
            },
            JsonField {
                name: "tags".to_string(),
                field_type: JsonFieldType::Array(Box::new(JsonFieldType::String)),
                description: Some("Array of string tags associated with the user".to_string()),
            },
        ],
        false, // not strict mode - allow extra fields
    );

    match agent_no_tools.call(nested_task).await {
        Ok(result) => {
            println!("Nested JSON Task Result: {}", result);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("Parsed JSON: {}", serde_json::to_string_pretty(&parsed)?);
                
                // Demonstrate accessing nested fields
                if let Some(personal_info) = parsed.get("personal_info") {
                    if personal_info.is_object() {
                        println!("✅ personal_info is correctly formatted as an object");
                        if let Some(name) = personal_info.get("name") {
                            println!("   - Name: {}", name);
                        }
                    }
                }
                if let Some(address) = parsed.get("address") {
                    if address.is_object() {
                        println!("✅ address is correctly formatted as an object");
                        if let Some(city) = address.get("city") {
                            println!("   - City: {}", city);
                        }
                    }
                }
            }
        },
        Err(e) => println!("Nested JSON Task Error: {}", e),
    }

    // Example 4: Array validation
    println!("\n=== Example 4: Array Validation ===");
    let array_task = Task::new_simple_json(
        "Create a list of 3 programming languages with their difficulty levels (1-10).".to_string(),
        Some("Programming languages with difficulty ratings in JSON format.".to_string()),
        vec![
            ("languages".to_string(), JsonFieldType::Array(Box::new(JsonFieldType::String))),
            ("difficulty_ratings".to_string(), JsonFieldType::Array(Box::new(JsonFieldType::Number))),
            ("total_count".to_string(), JsonFieldType::Number),
        ],
        true, // strict mode
    );

    match agent_no_tools.call(array_task).await {
        Ok(result) => {
            println!("Array Task Result: {}", result);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("Parsed JSON: {}", serde_json::to_string_pretty(&parsed)?);
                
                // Validate arrays
                if let Some(languages) = parsed.get("languages") {
                    if let Some(lang_array) = languages.as_array() {
                        println!("✅ languages array contains {} items", lang_array.len());
                    }
                }
                if let Some(ratings) = parsed.get("difficulty_ratings") {
                    if let Some(rating_array) = ratings.as_array() {
                        println!("✅ difficulty_ratings array contains {} items", rating_array.len());
                    }
                }
            }
        },
        Err(e) => println!("Array Task Error: {}", e),
    }

    // Test with tools (this will likely still fail until we fix the ChatMessage issue)
    println!("\n=== Example 5: Testing with Tools (Known Issue) ===");
    let tools = get_tools_by_names(&["get_current_time"]);
    let agent_with_tools = Agent::new(
        agent_llm_config,
        "You are a helpful assistant that can tell the time.".to_string(),
        vec!["Get current time using available tools.".to_string()],
        tools,
    );

    let tool_task = Task::new(
        "What time is it right now?".to_string(),
        Some("Current time information.".to_string()),
    );

    match agent_with_tools.call(tool_task).await {
        Ok(result) => println!("Tool Task Result: {}", result),
        Err(e) => println!("Tool Task Error (Expected): {}", e),
    }

    Ok(())
}
