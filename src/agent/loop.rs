use crate::{
  agent::TaskRequest,
  models::{
    ModelEvent, ModelProvider, ModelProviderError, ModelRequest, ModelResponse, ModelSelection,
    ModelToolCall,
  },
  tools::{ToolExecutionError, ToolName, ToolOutput, ToolRegistry},
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
    on_event: &mut dyn FnMut(AgentEvent),
  ) -> Result<AgentResult, AgentLoopError> {
    let mut request = ModelRequest::initial(task, model_selection.clone());

    for _ in 0..MAX_TURNS {
      let mut on_model_event = |event: ModelEvent| on_event(AgentEvent::Model(event));
      let response = match self.provider.respond(&request, &mut on_model_event) {
        Ok(response) => response,
        Err(error) => return Err(AgentLoopError::Model(error)),
      };

      match response {
        ModelResponse::Final(message) => return Ok(AgentResult { message }),
        ModelResponse::ToolCall(tool_call) => {
          let (tool_call_id, tool_request, arguments) = tool_call.into_parts();
          let tool_name = tool_request.name();
          let target = tool_request.target();
          on_event(AgentEvent::ToolCall {
            tool_name,
            target: target.clone(),
          });
          on_event(AgentEvent::ToolExecution { tool_name, target });
          let tool_output = match self.tools.execute(task, tool_request.clone()) {
            Ok(tool_output) => tool_output,
            Err(_) => ToolOutput::failure(),
          };
          let tool_call = ModelToolCall::new(tool_call_id, tool_request, arguments);
          request = request.with_tool_output(tool_call, tool_output);
        }
      }
    }

    return Err(AgentLoopError::TurnLimitExceeded);
  }
}

#[derive(Debug)]
pub enum AgentEvent {
  Model(ModelEvent),
  ToolCall { tool_name: ToolName, target: String },
  ToolExecution { tool_name: ToolName, target: String },
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
