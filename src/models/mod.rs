mod selection;

pub use selection::{ModelSelection, ModelSelectionError, ReasoningEffort};

use crate::{
  agent::TaskRequest,
  tools::{ToolName, ToolOutput, ToolRequest},
};

pub trait ModelProvider {
  fn respond(
    &self,
    request: ModelRequest,
    on_event: &mut dyn FnMut(ModelEvent),
  ) -> Result<ModelResponse, ModelProviderError>;
}

#[derive(Debug)]
pub enum ModelEvent {
  ThinkingStarted,
  TextDelta(String),
}

#[derive(Debug)]
pub struct ModelRequest {
  task: String,
  model_selection: ModelSelection,
  tool_output: Option<ToolOutput>,
  tool_call: Option<ModelToolCall>,
}

impl ModelRequest {
  pub fn initial(task: &TaskRequest, model_selection: ModelSelection) -> Self {
    return Self {
      task: task.task().to_owned(),
      model_selection,
      tool_output: None,
      tool_call: None,
    };
  }

  pub fn with_tool_output(
    task: &TaskRequest,
    model_selection: ModelSelection,
    tool_call: ModelToolCall,
    tool_output: ToolOutput,
  ) -> Self {
    return Self {
      task: task.task().to_owned(),
      model_selection,
      tool_output: Some(tool_output),
      tool_call: Some(tool_call),
    };
  }

  pub fn model_selection(&self) -> &ModelSelection {
    return &self.model_selection;
  }

  pub fn task(&self) -> &str {
    return &self.task;
  }

  pub fn tool_output(&self) -> Option<&ToolOutput> {
    return self.tool_output.as_ref();
  }

  pub fn tool_call(&self) -> Option<&ModelToolCall> {
    return self.tool_call.as_ref();
  }
}

#[derive(Debug)]
pub struct ModelToolCall {
  id: String,
  request: ToolRequest,
  arguments: String,
}

impl ModelToolCall {
  pub fn new(id: String, request: ToolRequest, arguments: String) -> Self {
    return Self {
      id,
      request,
      arguments,
    };
  }

  pub fn id(&self) -> &str {
    return &self.id;
  }

  pub fn arguments(&self) -> &str {
    return &self.arguments;
  }

  pub fn tool_name(&self) -> ToolName {
    let tool_name = self.request.name();

    return tool_name;
  }

  pub fn into_parts(self) -> (String, ToolRequest, String) {
    return (self.id, self.request, self.arguments);
  }
}

#[derive(Debug)]
pub enum ModelResponse {
  Final(String),
  ToolCall(ModelToolCall),
}

#[derive(Debug)]
pub enum ModelProviderError {
  Exhausted,
  MissingCredentials,
  NotImplemented,
  RequestFailed,
  InvalidResponse,
  UnsupportedToolCall,
}
