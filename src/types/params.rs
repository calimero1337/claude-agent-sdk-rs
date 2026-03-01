//! Request parameter types.

use crate::types::message::Message;
use crate::types::tool::{Tool, ToolChoice};

/// Maximum number of agent turns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaxTurns {
    Unlimited,
    Limited(usize),
}

impl Default for MaxTurns {
    fn default() -> Self {
        Self::Limited(10)
    }
}

impl MaxTurns {
    /// Returns true if the given turn count has reached or exceeded the limit.
    pub fn is_exceeded(&self, turns: usize) -> bool {
        match self {
            MaxTurns::Unlimited => false,
            MaxTurns::Limited(max) => turns >= *max,
        }
    }
}

/// Parameters for creating an agent session.
#[derive(Debug, Clone)]
pub struct AgentParams {
    /// The initial task/prompt for the agent.
    pub prompt: String,
    /// The Claude model to use.
    pub model: String,
    /// Maximum number of turns the agent can take.
    pub max_turns: MaxTurns,
    /// System prompt.
    pub system: Option<String>,
    /// Tools available to the agent.
    pub tools: Vec<Tool>,
    /// How the model should choose tools.
    pub tool_choice: ToolChoice,
    /// Maximum tokens to generate per turn.
    pub max_tokens: u32,
    /// Initial conversation history.
    pub messages: Vec<Message>,
}

impl AgentParams {
    /// Create a new [`AgentParamsBuilder`].
    pub fn builder(prompt: impl Into<String>) -> AgentParamsBuilder {
        AgentParamsBuilder::new(prompt)
    }
}

impl Default for AgentParams {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            model: "claude-sonnet-4-6".to_string(),
            max_turns: MaxTurns::default(),
            system: None,
            tools: Vec::new(),
            tool_choice: ToolChoice::Auto,
            max_tokens: 16384,
            messages: Vec::new(),
        }
    }
}

/// Builder for [`AgentParams`].
pub struct AgentParamsBuilder {
    params: AgentParams,
}

impl AgentParamsBuilder {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            params: AgentParams {
                prompt: prompt.into(),
                ..AgentParams::default()
            },
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.params.model = model.into();
        self
    }

    pub fn max_turns(mut self, max_turns: MaxTurns) -> Self {
        self.params.max_turns = max_turns;
        self
    }

    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.params.system = Some(system.into());
        self
    }

    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.params.tools = tools;
        self
    }

    pub fn tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.params.tool_choice = tool_choice;
        self
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.params.max_tokens = max_tokens;
        self
    }

    pub fn build(self) -> AgentParams {
        self.params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_turns_exceeded() {
        assert!(!MaxTurns::Unlimited.is_exceeded(1000));
        assert!(MaxTurns::Limited(5).is_exceeded(5));
        assert!(MaxTurns::Limited(5).is_exceeded(6));
        assert!(!MaxTurns::Limited(5).is_exceeded(4));
        assert!(MaxTurns::Limited(0).is_exceeded(0));
    }

    #[test]
    fn agent_params_builder() {
        let params = AgentParams::builder("Do something")
            .model("claude-sonnet-4-6")
            .max_turns(MaxTurns::Limited(20))
            .max_tokens(4096)
            .build();

        assert_eq!(params.prompt, "Do something");
        assert_eq!(params.model, "claude-sonnet-4-6");
        assert_eq!(params.max_tokens, 4096);
        assert_eq!(params.max_turns, MaxTurns::Limited(20));
    }

    #[test]
    fn agent_params_default_model() {
        let params = AgentParams::builder("test").build();
        assert_eq!(params.model, "claude-sonnet-4-6");
        assert_eq!(params.max_tokens, 16384);
    }

    #[test]
    fn agent_params_system_prompt() {
        let params = AgentParams::builder("task")
            .system("You are a helpful assistant")
            .build();
        assert_eq!(
            params.system.as_deref(),
            Some("You are a helpful assistant")
        );
    }
}
