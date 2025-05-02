

use merco_agents::agent::agent::Agent;
use merco_agents::llm::interface::LLMConfig;
use merco_agents::task::task::Task;



#[tokio::main]
async fn main() {
    let llm_config = LLMConfig::new(
        "ollama".to_string(),
        "qwen3:4b".to_string(),
        None,
        "http://localhost:11434".to_string(),
    );

    let agent = Agent::new(llm_config, "You are a helpful assistant".to_string(), vec!["You are a helpful assistant".to_string()], vec![]);

    let task = Task::new("You are a helpful assistant".to_string(), None);

    let result = agent.run(task).await;

    println!("Result: {:?}", result);
}