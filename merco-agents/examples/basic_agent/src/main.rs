use merco_agents::agent::agent::{Agent, AgentLLMConfig};
use merco_agents::task::task::Task;
use merco_llmproxy::{LlmConfig, Provider};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv().ok();

    println!("🤖 Basic Agent Example");
    println!("====================");

    // Get API key
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("Please set OPENROUTER_API_KEY environment variable");

    // Configure LLM
    let llm_config = LlmConfig::new(Provider::OpenAI)
        .with_base_url("https://openrouter.ai/api/v1".to_string())
        .with_api_key(api_key);

    let agent_llm_config = AgentLLMConfig::new(
        llm_config, 
        "openai/gpt-4o-mini".to_string(), 
        0.0, 
        1000
    );

    // Create an agent with a specific role
    let writing_agent = Agent::new(
        agent_llm_config,
        "You are a creative writing assistant that helps create engaging content.".to_string(),
        vec![
            "Write clear and engaging content".to_string(),
            "Maintain a professional yet friendly tone".to_string(),
            "Be creative but accurate".to_string(),
        ],
        vec![], // No tools for this basic example
    );

    // Example 1: Simple question
    println!("\n📝 Example 1: Simple Question");
    let task1 = Task::new(
        "Explain what artificial intelligence is in simple terms.".to_string(),
        Some("A clear, beginner-friendly explanation in 2-3 sentences.".to_string()),
    );

    match writing_agent.call(task1).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    // Example 2: Creative task
    println!("\n🎨 Example 2: Creative Writing");
    let task2 = Task::new(
        "Write a short story opening about a robot who discovers emotions.".to_string(),
        Some("An engaging story opening of 3-4 sentences.".to_string()),
    );

    match writing_agent.call(task2).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    // Example 3: Problem-solving task
    println!("\n🧠 Example 3: Problem Solving");
    let task3 = Task::new(
        "You have 3 apples and give away 1. Then someone gives you 2 more apples. How many apples do you have now? Explain your reasoning.".to_string(),
        Some("Clear mathematical reasoning with the final answer.".to_string()),
    );

    match writing_agent.call(task3).await {
        Ok(result) => {
            println!("✅ Result:");
            println!("{}", result);
        },
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n🎉 Basic Agent Example Complete!");
    Ok(())
}
