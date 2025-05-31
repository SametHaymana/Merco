use merco_agents::agent::agent::{Agent, AgentLLMConfig};
use merco_agents::task::task::{Task, JsonFieldType};
use merco_llmproxy::{LlmConfig, Provider, get_tools_by_names, merco_tool};
use dotenv::dotenv;
use chrono::prelude::*;

// Tool 1: Get current time
#[merco_tool(description = "Get the current date and time")]
pub fn get_current_time() -> String {
    let now: DateTime<Local> = Local::now();
    now.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}

// Tool 2: Calculate simple math
#[merco_tool(description = "Calculate basic mathematical operations (add, subtract, multiply, divide). Format: 'operation,number1,number2' e.g., 'add,5,3'")]
pub fn calculate(expression: String) -> String {
    let parts: Vec<&str> = expression.split(',').collect();
    if parts.len() != 3 {
        return "Error: Please provide operation,number1,number2".to_string();
    }
    
    let operation = parts[0].trim();
    let num1: f64 = match parts[1].trim().parse() {
        Ok(n) => n,
        Err(_) => return format!("Error: '{}' is not a valid number", parts[1]),
    };
    let num2: f64 = match parts[2].trim().parse() {
        Ok(n) => n,
        Err(_) => return format!("Error: '{}' is not a valid number", parts[2]),
    };
    
    let result = match operation {
        "add" => num1 + num2,
        "subtract" => num1 - num2,
        "multiply" => num1 * num2,
        "divide" => {
            if num2 == 0.0 {
                return "Error: Division by zero".to_string();
            }
            num1 / num2
        },
        _ => return format!("Error: Unknown operation '{}'. Use add, subtract, multiply, or divide", operation),
    };
    
    format!("{} {} {} = {}", num1, operation, num2, result)
}

// Tool 3: Generate random number
#[merco_tool(description = "Generate a random number between min and max (inclusive). Format: 'min,max' e.g., '1,100'")]
pub fn random_number(range: String) -> String {
    let parts: Vec<&str> = range.split(',').collect();
    if parts.len() != 2 {
        return "Error: Please provide min,max".to_string();
    }
    
    let min: i32 = match parts[0].trim().parse() {
        Ok(n) => n,
        Err(_) => return format!("Error: '{}' is not a valid number", parts[0]),
    };
    let max: i32 = match parts[1].trim().parse() {
        Ok(n) => n,
        Err(_) => return format!("Error: '{}' is not a valid number", parts[1]),
    };
    
    if min > max {
        return "Error: min cannot be greater than max".to_string();
    }
    
    let random_num = min + (rand::random::<u32>() % (max - min + 1) as u32) as i32;
    format!("Random number between {} and {}: {}", min, max, random_num)
}

// Tool 4: Text analysis
#[merco_tool(description = "Analyze text and return word count, character count, and sentence count")]
pub fn analyze_text(text: String) -> String {
    let word_count = text.split_whitespace().count();
    let char_count = text.chars().count();
    let sentence_count = text.split(['.', '!', '?']).filter(|s| !s.trim().is_empty()).count();
    
    format!(
        "Text Analysis: {} characters, {} words, {} sentences",
        char_count, word_count, sentence_count
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    println!("🛠️  Tool Usage Example");
    println!("=====================");

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

    // Get available tools
    let tools = get_tools_by_names(&[
        "get_current_time",
        "calculate", 
        "random_number",
        "analyze_text"
    ]);

    let assistant_agent = Agent::new(
        agent_llm_config,
        "You are a helpful assistant with access to various tools. Use the tools when needed to provide accurate information.".to_string(),
        vec![
            "Use tools to get accurate, real-time information".to_string(),
            "Always call the appropriate tool when the user asks for calculations, time, random numbers, or text analysis".to_string(),
            "Provide clear, helpful responses based on tool results".to_string(),
        ],
        tools,
    );

    // Example 1: Time-based task
    println!("\n⏰ Example 1: Current Time Query");
    let time_task = Task::new(
        "What time is it right now? Please include the date as well.".to_string(),
        Some("Current date and time information.".to_string()),
    );

    match assistant_agent.call(time_task).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    // Example 2: Mathematical calculation
    println!("\n🧮 Example 2: Mathematical Calculation");
    let math_task = Task::new(
        "Calculate 25 multiplied by 8, then add 17 to the result.".to_string(),
        Some("Step-by-step calculation with final result.".to_string()),
    );

    match assistant_agent.call(math_task).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    // Example 3: Random number generation with JSON output
    println!("\n🎲 Example 3: Random Number Generator (JSON Format)");
    let random_task = Task::new_simple_json(
        "Generate 3 random numbers between 1 and 100, and tell me the current time.".to_string(),
        Some("Random numbers and time information in structured format.".to_string()),
        vec![
            ("current_time".to_string(), JsonFieldType::String),
            ("random_numbers".to_string(), JsonFieldType::Array(Box::new(JsonFieldType::Number))),
            ("range_min".to_string(), JsonFieldType::Number),
            ("range_max".to_string(), JsonFieldType::Number),
            ("count".to_string(), JsonFieldType::Number),
        ],
        true, // strict mode
    );

    match assistant_agent.call(random_task).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("\n🔍 JSON validation passed!");
                println!("📦 Pretty printed:");
                println!("{}", serde_json::to_string_pretty(&parsed)?);
            }
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    // Example 4: Text analysis
    println!("\n📝 Example 4: Text Analysis");
    let text_task = Task::new(
        "Analyze this text: 'Artificial intelligence is transforming the world. It helps us solve complex problems. The future looks very promising!'".to_string(),
        Some("Text analysis results with counts and insights.".to_string()),
    );

    match assistant_agent.call(text_task).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    // Example 5: Multi-tool usage with JSON output
    println!("\n🎯 Example 5: Multi-Tool Analysis (JSON Format)");
    let multi_task = Task::new_simple_json(
        "Create a report with: current time, a random number between 1-10, the result of 15 divided by 3, and analysis of the text 'Hello world! How are you today?'".to_string(),
        Some("Complete report using multiple tools in JSON format.".to_string()),
        vec![
            ("timestamp".to_string(), JsonFieldType::String),
            ("random_number".to_string(), JsonFieldType::Number),
            ("division_result".to_string(), JsonFieldType::Number),
            ("text_analysis".to_string(), JsonFieldType::Object),
            ("report_generated".to_string(), JsonFieldType::Boolean),
        ],
        true, // strict mode
    );

    match assistant_agent.call(multi_task).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("\n🔍 Multi-tool JSON validation passed!");
                println!("📦 Pretty printed:");
                println!("{}", serde_json::to_string_pretty(&parsed)?);
            }
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n🎉 Tool Usage Example Complete!");
    println!("Demonstrated features:");
    println!("  ✅ Multiple custom tools integration");
    println!("  ✅ Time and date tools");
    println!("  ✅ Mathematical calculation tools");
    println!("  ✅ Random number generation");
    println!("  ✅ Text analysis tools");
    println!("  ✅ JSON output with tool results");
    println!("  ✅ Multi-tool usage in single tasks");
    
    Ok(())
}
