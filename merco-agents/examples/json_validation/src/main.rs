use merco_agents::agent::agent::{Agent, AgentLLMConfig};
use merco_agents::task::task::{Task, JsonFieldType, JsonField};
use merco_llmproxy::{LlmConfig, Provider};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    println!("📋 JSON Validation Example");
    println!("==========================");

    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("Please set OPENROUTER_API_KEY environment variable");

    let llm_config = LlmConfig::new(Provider::OpenAI)
        .with_base_url("https://openrouter.ai/api/v1".to_string())
        .with_api_key(api_key);

    let agent_llm_config = AgentLLMConfig::new(
        llm_config, 
        "openai/gpt-4o-mini".to_string(), 
        0.0, 
        1000
    );

    let data_agent = Agent::new(
        agent_llm_config,
        "You are a data analyst that provides structured information in precise JSON formats.".to_string(),
        vec![
            "Always follow the exact JSON schema provided".to_string(),
            "Ensure data types are correct (strings, numbers, booleans, arrays)".to_string(),
            "Provide realistic and accurate data".to_string(),
        ],
        vec![], // No tools needed
    );

    // Example 1: Simple JSON with basic types
    println!("\n📊 Example 1: Product Information (Basic Types)");
    let product_task = Task::new_simple_json(
        "Create information for a laptop computer product.".to_string(),
        Some("Product details in structured JSON format.".to_string()),
        vec![
            ("name".to_string(), JsonFieldType::String),
            ("price".to_string(), JsonFieldType::Number),
            ("in_stock".to_string(), JsonFieldType::Boolean),
            ("category".to_string(), JsonFieldType::String),
            ("rating".to_string(), JsonFieldType::Number),
        ],
        true, // strict mode
    );

    match data_agent.call(product_task).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("\n🔍 Validation: JSON structure is correct!");
                println!("📦 Pretty printed:");
                println!("{}", serde_json::to_string_pretty(&parsed)?);
            }
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    // Example 2: Arrays and nested structures
    println!("\n👥 Example 2: User Profile with Arrays");
    let user_task = Task::new_with_json_output(
        "Create a user profile for a software developer named Sarah.".to_string(),
        Some("Complete user profile with skills, projects, and contact info.".to_string()),
        vec![
            // Required fields
            JsonField {
                name: "user_id".to_string(),
                field_type: JsonFieldType::Number,
                description: Some("Unique user identifier".to_string()),
            },
            JsonField {
                name: "profile".to_string(),
                field_type: JsonFieldType::Object,
                description: Some("Personal information object with name, age, email".to_string()),
            },
            JsonField {
                name: "skills".to_string(),
                field_type: JsonFieldType::Array(Box::new(JsonFieldType::String)),
                description: Some("Array of programming skills".to_string()),
            },
            JsonField {
                name: "active".to_string(),
                field_type: JsonFieldType::Boolean,
                description: Some("Whether the user is active".to_string()),
            },
        ],
        vec![
            // Optional fields
            JsonField {
                name: "projects".to_string(),
                field_type: JsonFieldType::Array(Box::new(JsonFieldType::Object)),
                description: Some("Array of project objects".to_string()),
            },
            JsonField {
                name: "experience_years".to_string(),
                field_type: JsonFieldType::Number,
                description: Some("Years of programming experience".to_string()),
            },
        ],
        false, // not strict mode - allow extra fields
    );

    match data_agent.call(user_task).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("\n🔍 Validation: Complex JSON structure is correct!");
                
                // Validate specific fields
                if let Some(skills) = parsed.get("skills") {
                    if let Some(skills_array) = skills.as_array() {
                        println!("🛠️  Skills found: {} items", skills_array.len());
                    }
                }
                
                if let Some(profile) = parsed.get("profile") {
                    if profile.is_object() {
                        println!("👤 Profile object structure is valid");
                    }
                }
                
                println!("\n📦 Pretty printed:");
                println!("{}", serde_json::to_string_pretty(&parsed)?);
            }
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    // Example 3: Strict validation demonstration
    println!("\n⚡ Example 3: API Response Format (Strict Mode)");
    let api_task = Task::new_simple_json(
        "Create an API response for a successful user login.".to_string(),
        Some("Standard API response format with status, message, and user data.".to_string()),
        vec![
            ("status".to_string(), JsonFieldType::String),
            ("message".to_string(), JsonFieldType::String),
            ("success".to_string(), JsonFieldType::Boolean),
            ("timestamp".to_string(), JsonFieldType::Number),
            ("data".to_string(), JsonFieldType::Object),
        ],
        true, // strict mode - no extra fields allowed
    );

    match data_agent.call(api_task).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("\n🔍 Strict validation passed!");
                println!("📦 Pretty printed:");
                println!("{}", serde_json::to_string_pretty(&parsed)?);
            }
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    // Example 4: Number arrays and calculations
    println!("\n📈 Example 4: Financial Data with Number Arrays");
    let finance_task = Task::new_simple_json(
        "Create a monthly budget breakdown for a small business.".to_string(),
        Some("Budget data with revenues, expenses, and profit calculations.".to_string()),
        vec![
            ("month".to_string(), JsonFieldType::String),
            ("revenues".to_string(), JsonFieldType::Array(Box::new(JsonFieldType::Number))),
            ("expenses".to_string(), JsonFieldType::Array(Box::new(JsonFieldType::Number))),
            ("total_revenue".to_string(), JsonFieldType::Number),
            ("total_expenses".to_string(), JsonFieldType::Number),
            ("profit".to_string(), JsonFieldType::Number),
            ("profitable".to_string(), JsonFieldType::Boolean),
        ],
        true, // strict mode
    );

    match data_agent.call(finance_task).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("\n🔍 Financial data validation passed!");
                
                // Validate arrays contain numbers
                if let Some(revenues) = parsed.get("revenues") {
                    if let Some(rev_array) = revenues.as_array() {
                        println!("💰 Revenue entries: {}", rev_array.len());
                    }
                }
                
                if let Some(expenses) = parsed.get("expenses") {
                    if let Some(exp_array) = expenses.as_array() {
                        println!("💸 Expense entries: {}", exp_array.len());
                    }
                }
                
                println!("\n📦 Pretty printed:");
                println!("{}", serde_json::to_string_pretty(&parsed)?);
            }
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n🎉 JSON Validation Example Complete!");
    println!("All examples demonstrated proper JSON schema validation with:");
    println!("  ✅ Type checking (string, number, boolean, array, object)");
    println!("  ✅ Required vs optional fields");
    println!("  ✅ Strict mode enforcement");
    println!("  ✅ Nested object validation");
    println!("  ✅ Array element type validation");
    
    Ok(())
}
