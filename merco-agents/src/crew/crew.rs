use crate::agent::agent::Agent;
use crate::task::task::Task;
use anyhow::{anyhow, Context, Result};
use std::sync::Arc;

// Enum to define the workflow execution strategy
#[derive(Debug, Clone, PartialEq)]
pub enum Workflow {
    Sequential,
    Hierarchical, // Placeholder for now
}

// The Crew struct
#[derive(Debug, Clone)]
pub struct Crew {
    agents: Vec<Arc<Agent>>, // Use Arc for shared ownership if needed, especially for hierarchical
    tasks: Vec<Task>,
    workflow: Workflow,
    // manager_agent: Option<Arc<Agent>>, // Optional for hierarchical planning
    // manager_llm_config: Option<AgentLLMConfig>, // Optional for hierarchical planning
}

impl Crew {
    pub fn new(agents: Vec<Arc<Agent>>, tasks: Vec<Task>, workflow: Workflow) -> Self {
        // Basic validation: For sequential, number of agents often matches tasks, 
        // but maybe one agent handles multiple tasks. Let's allow flexibility for now.
        // Hierarchical validation would be different.
        // assert_eq!(agents.len(), tasks.len(), "Sequential workflow requires one agent per task (for now).");
        
        Self {
            agents,
            tasks,
            workflow,
        }
    }

    // Main execution entry point
    pub async fn run(&self) -> Result<String> {
        match self.workflow {
            Workflow::Sequential => self.run_sequential().await,
            Workflow::Hierarchical => self.run_hierarchical().await, // To be implemented
        }
    }

    // --- Sequential Workflow Implementation ---
    async fn run_sequential(&self) -> Result<String> {
        if self.agents.is_empty() || self.tasks.is_empty() {
            return Ok("No agents or tasks to run.".to_string());
        }

        let mut results = Vec::new();
        let mut current_task_output: Option<String> = None;

        // Simple sequential: Assume one agent executes all tasks in order,
        // or pair agents with tasks sequentially if counts match.
        // For simplicity now, let's assume the first agent runs all tasks, 
        // feeding output to the next task's context.
        // A more robust implementation would explicitly pair agents and tasks.
        
        let agent = self.agents[0].clone(); // Use the first agent for all tasks for now

        for task in &self.tasks {
            let mut current_task = task.clone(); // Clone task to modify description

            // Inject previous output
            if let Some(ref output) = current_task_output {
                current_task.description = format!(
                    "Previous Task Output:\n{}
\n---\n\nOriginal Task:\n{}",
                    output,
                    task.description // Keep original desc for context message
                );
            }

            println!("\nRunning Task: {} by Agent...", current_task.description.lines().next().unwrap_or_default());
            
            // Call agent, convert error, and add context
            let result = agent.call(current_task.clone()) 
                .await
                .map_err(|e| anyhow!(e)) 
                // Simplified context message referencing the cloned task's description
                .with_context(|| format!("Agent failed to execute task starting with: '{}'", current_task.description.chars().take(50).collect::<String>()))?;
                
            println!("Task Result: {}", result);
            current_task_output = Some(result.clone()); 
            results.push(result);
        }

        // Return the output of the last task for sequential workflow
        Ok(current_task_output.unwrap_or_else(|| "Sequential run completed with no output.".to_string()))
    }
    
    // --- Hierarchical Workflow Implementation (Placeholder) ---
    async fn run_hierarchical(&self) -> Result<String> {
        // 1. Planning Phase (Requires a Manager Agent/LLM call)
        //    - Define overall goal.
        //    - Manager analyzes goal, agents, tasks -> Creates an execution plan (DAG?)
        println!("Hierarchical workflow planning started (Not Implemented).");
        // let plan = self.plan_execution().await?;
        
        // 2. Execution Phase (Based on Plan)
        //    - Execute tasks according to the plan (handle dependencies, parallelism).
        println!("Hierarchical workflow execution started (Not Implemented).");
        // let execution_results = self.execute_plan(plan).await?;
        
        // 3. Synthesis Phase (Requires Manager Agent/LLM call)
        //    - Manager synthesizes results into a final output.
        println!("Hierarchical workflow synthesis started (Not Implemented).");
        // let final_output = self.synthesize_results(execution_results).await?;

        // Placeholder result
        Ok("Hierarchical workflow not fully implemented.".to_string())
    }
}

