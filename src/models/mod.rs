use crate::{
  agent::TaskRequest,
  tools::{ToolOutput, ToolRequest},
};

pub trait ModelProvider {
  fn respond(&self, request: ModelRequest) -> Result<ModelResponse, ModelProviderError>;
}

#[derive(Debug)]
pub struct ModelRequest {
  task: String,
  tool_output: Option<ToolOutput>,
}

impl ModelRequest {
  pub fn initial(task: &TaskRequest) -> Self {
    return Self {
      task: task.task().to_owned(),
      tool_output: None,
    };
  }

  pub fn with_tool_output(task: &TaskRequest, tool_output: ToolOutput) -> Self {
    return Self {
      task: task.task().to_owned(),
      tool_output: Some(tool_output),
    };
  }

  pub fn task(&self) -> &str {
    return &self.task;
  }

  pub fn tool_output(&self) -> Option<&ToolOutput> {
    return self.tool_output.as_ref();
  }
}

#[derive(Debug)]
pub enum ModelResponse {
  Final(String),
  ToolCall(ToolRequest),
}

#[derive(Debug)]
pub enum ModelProviderError {
  Exhausted,
  MissingCredentials,
  NotImplemented,
}
