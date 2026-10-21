use crate::{
  agent::TaskRequest,
  models::{ModelProvider, ModelProviderError, ModelRequest, ModelResponse, ModelSelection},
  tools::{ToolExecutionError, ToolRegistry},
};

const MAX_TURNS: usize = 16;

pub struct AgentLoop<P> {
  provider: P,
  tools: ToolRegistry,
}

impl<P: ModelProvider> AgentLoop<P> {
  pub fn new(provider: P, tools: ToolRegistry) -> Self {
    return Self { provider, tools };
  }

  pub fn execute(
    &self,
    task: &TaskRequest,
    model_selection: &ModelSelection,
  ) -> Result<AgentResult, AgentLoopError> {
    let mut request = ModelRequest::initial(task, model_selection.clone());

    for _ in 0..MAX_TURNS {
      let response = match self.provider.respond(request) {
        Ok(response) => response,
        Err(error) => return Err(AgentLoopError::Model(error)),
      };

      match response {
        ModelResponse::Final(message) => return Ok(AgentResult { message }),
        ModelResponse::ToolCall(tool_request) => {
          let tool_output = match self.tools.execute(task, tool_request) {
            Ok(tool_output) => tool_output,
            Err(error) => return Err(AgentLoopError::Tool(error)),
          };
          request = ModelRequest::with_tool_output(task, model_selection.clone(), tool_output);
        }
      }
    }

    return Err(AgentLoopError::TurnLimitExceeded);
  }
}

#[derive(Debug)]
pub struct AgentResult {
  message: String,
}

impl AgentResult {
  pub fn message(&self) -> &str {
    return &self.message;
  }
}

#[derive(Debug)]
pub enum AgentLoopError {
  Model(ModelProviderError),
  Tool(ToolExecutionError),
  TurnLimitExceeded,
}
