use merco_agents::agent::agent::Agent;
use merco_agents::task::task::Task;
use merco_llmproxy::{LlmConfig, Provider, get_tools_by_names, merco_tool};
use chrono::prelude::*;
use merco_agents::agent::agent::AgentLLMConfig;

use dotenv::dotenv;

#[merco_tool(description = "A tool to get the current time")]
pub fn get_current_time() -> String {
    println!("get_current_time");
    
    // Get the current system time
    let now = std::time::SystemTime::now();
    // Convert SystemTime to DateTime<Local> using the chrono crate
    // (Requires adding `chrono = { version = "0.4", features = ["serde"] }` to Cargo.toml and `use chrono::prelude::*;` potentially)
    let datetime: chrono::DateTime<chrono::Local> = now.into();
    // Format the datetime into a human-readable string
    datetime.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");

    let llm_config = LlmConfig::new(Provider::OpenAI)
        .with_base_url("https://openrouter.ai/api/v1".to_string())
        .with_api_key(api_key);

    let agent_llm_config = AgentLLMConfig::new(llm_config, "openai/gpt-4o-mini".to_string(), 0.0, 1000);

    let tools = get_tools_by_names(&["get_current_time"]);
    let agent = Agent::new(
        agent_llm_config,
        "You are a helpful assistant".to_string(),
        vec!["You are a helpful assistant".to_string()],
        tools,
    );

    let task = Task::new(
        "What time is it?".to_string(),
        Some("A verbose markdown formatted output!".to_string()),
    );

    let result = agent.call(task).await;

    println!("Result: {:?}", result);
}
