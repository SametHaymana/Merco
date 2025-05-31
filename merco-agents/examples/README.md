# Merco Agents Examples

This directory contains various examples demonstrating the capabilities of the Merco Agents framework. Each example showcases different features and use cases.

## Prerequisites

Before running any examples, make sure you have:

1. **Rust installed** (latest stable version)
2. **Environment variables set up**:
   ```bash
   export OPENROUTER_API_KEY="your-openrouter-api-key"
   ```
   Or create a `.env` file in each example directory:
   ```
   OPENROUTER_API_KEY=your-openrouter-api-key
   ```

## Examples Overview

### 1. 🤖 Basic Agent (`basic_agent/`)

**What it demonstrates:**
- Basic agent creation and configuration
- Simple text-based tasks
- Different types of prompts (creative writing, problem-solving, explanations)

**Features shown:**
- Agent initialization with backstory and goals
- Text output tasks
- Basic error handling

**Run it:**
```bash
cd basic_agent
cargo run
```

**What you'll see:**
- Simple Q&A interactions
- Creative writing examples
- Mathematical reasoning tasks

---

### 2. 📋 JSON Validation (`json_validation/`)

**What it demonstrates:**
- Structured JSON output with schema validation
- Type checking (strings, numbers, booleans, arrays, objects)
- Required vs optional fields
- Strict vs non-strict validation modes

**Features shown:**
- Simple JSON schemas with basic types
- Complex nested objects and arrays
- Strict mode enforcement
- JSON parsing and pretty-printing

**Run it:**
```bash
cd json_validation
cargo run
```

**What you'll see:**
- Product information in JSON format
- User profiles with nested objects
- API response formats
- Financial data with number arrays

---

### 3. 🛠️ Tool Usage (`tool_usage/`)

**What it demonstrates:**
- Custom tool integration
- Tool calling with function execution
- Combining tools with JSON output validation
- Multi-tool usage in single tasks

**Available Tools:**
- `get_current_time`: Get current date and time
- `calculate`: Basic math operations (add, subtract, multiply, divide)
- `random_number`: Generate random numbers in a range
- `analyze_text`: Text analysis (word/character/sentence count)

**Features shown:**
- Creating custom tools with `#[merco_tool]` macro
- Tool parameter handling and validation
- Error handling in tools
- JSON output combined with tool results

**Run it:**
```bash
cd tool_usage
cargo run
```

**What you'll see:**
- Time queries using tools
- Mathematical calculations
- Random number generation with JSON output
- Text analysis results
- Multi-tool reports in structured JSON

---

## Example Output Formats

### Text Output
Simple string responses for basic tasks:
```
Artificial intelligence (AI) is technology that enables machines to simulate human intelligence...
```

### JSON Output
Structured data with validation:
```json
{
  "name": "Gaming Laptop Pro",
  "price": 1299.99,
  "in_stock": true,
  "category": "Electronics",
  "rating": 4.5
}
```

### Tool Integration
Combining tool results with structured output:
```json
{
  "timestamp": "2024-01-15 14:30:25 PST",
  "random_number": 7,
  "division_result": 5.0,
  "text_analysis": {
    "characters": 26,
    "words": 6,
    "sentences": 2
  },
  "report_generated": true
}
```

## Common Patterns

### Basic Agent Setup
```rust
use merco_agents::agent::agent::{Agent, AgentLLMConfig};
use merco_llmproxy::{LlmConfig, Provider};

let llm_config = LlmConfig::new(Provider::OpenAI)
    .with_base_url("https://openrouter.ai/api/v1".to_string())
    .with_api_key(api_key);

let agent_llm_config = AgentLLMConfig::new(
    llm_config, 
    "openai/gpt-4o-mini".to_string(), 
    0.0, 
    1000
);

let agent = Agent::new(
    agent_llm_config,
    "You are a helpful assistant...".to_string(),
    vec!["Goal 1".to_string(), "Goal 2".to_string()],
    tools, // Vec<Tool>
);
```

### Creating Tasks
```rust
use merco_agents::task::task::{Task, JsonFieldType};

// Text task
let task = Task::new(
    "Your question here".to_string(),
    Some("Expected output description".to_string()),
);

// JSON task
let json_task = Task::new_simple_json(
    "Your question here".to_string(),
    Some("Expected output description".to_string()),
    vec![
        ("field_name".to_string(), JsonFieldType::String),
        ("count".to_string(), JsonFieldType::Number),
    ],
    true, // strict mode
);
```

### Creating Tools
```rust
use merco_llmproxy::merco_tool;

#[merco_tool(description = "Tool description for the LLM")]
pub fn my_tool(parameter: String) -> String {
    // Tool implementation
    format!("Result: {}", parameter)
}
```

## Configuration

### Model Selection
All examples use `"openai/gpt-4o-mini"` by default, but you can change this to:
- `"openai/gpt-4"` (more capable, more expensive)
- `"anthropic/claude-3-sonnet"` (alternative provider)
- Any model supported by OpenRouter

### Temperature Settings
- `0.0` - Deterministic, consistent outputs (used in examples)
- `0.7` - More creative, varied outputs
- `1.0` - Very creative, potentially less coherent

### Token Limits
- `1000` tokens used in examples (sufficient for most tasks)
- Increase for longer outputs
- Monitor costs with higher limits

## Troubleshooting

### Common Issues

1. **API Key not set**
   ```
   Error: Please set OPENROUTER_API_KEY environment variable
   ```
   Solution: Set the environment variable or create a `.env` file

2. **JSON Validation Errors**
   ```
   Output validation failed: Missing required field 'name'
   ```
   Solution: The agent will automatically retry up to 3 times with feedback

3. **Tool Execution Errors**
   ```
   Tool Execution Error: Error: 'abc' is not a valid number
   ```
   Solution: Tools handle errors gracefully and return error messages

4. **LLM Request Failures**
   ```
   LLM execution failed: HTTP 401 Unauthorized
   ```
   Solution: Check your API key and account balance

### Debug Tips

1. **Enable detailed logging**: Set `RUST_LOG=debug` environment variable
2. **Check API usage**: Monitor your OpenRouter dashboard
3. **Validate JSON manually**: Use online JSON validators if needed
4. **Test tools separately**: Call tool functions directly in unit tests

## Next Steps

After running these examples, you can:

1. **Create custom agents** for your specific use cases
2. **Build domain-specific tools** for your applications
3. **Design complex JSON schemas** for structured data
4. **Integrate with your existing Rust applications**

## Support

For issues or questions:
- Check the main project documentation
- Review the source code in `src/`
- Look at the test cases for more examples

Happy coding with Merco Agents! 🚀 