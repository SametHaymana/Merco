

use merco_agents::agent::agent::Agent;
use merco_agents::llm::interface::LLMConfig;
use merco_agents::task::task::Task;



#[tokio::main]
async fn main() {
    //let key = "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";

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