

use merco_agents::agent::agent::Agent;
use merco_agents::task::task::Task;
use merco_llmproxy::{get_tools_by_names, merco_tool, LlmConfig, Provider};

use merco_agents::agent::agent::AgentLLMConfig;

use dotenv::dotenv;


#[merco_tool(description = "A tool to get the current time")]
pub fn get_current_time() -> String {
    let now = std::time::SystemTime::now();
    let timestamp = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    timestamp.as_secs().to_string()
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");

    println!("API Key: {}", api_key);

    let llm_config = LlmConfig::new(
        Provider::OpenAI,
    )
    .with_base_url("https://openrouter.ai/api/v1".to_string())
    .with_api_key(api_key);

    let agent_llm_config = 
    AgentLLMConfig::new(llm_config, "gpt-4o-mini".to_string(), 0.0, 1000);


    let tools = get_tools_by_names(&["get_current_time"]);
    let agent = Agent::new(agent_llm_config, "You are a helpful assistant".to_string(), vec!["You are a helpful assistant".to_string()], tools);

    let task = Task::new("What is the current time?".to_string(), Some("A json output!".to_string()));

    let result = agent.call(task).await;

    println!("Result: {:?}", result);
}