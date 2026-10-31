mod selection;

pub use selection::{ModelSelection, ModelSelectionError, ReasoningEffort};

use crate::{
  agent::TaskRequest,
  tools::{ToolName, ToolOutput, ToolRequest},
};

pub trait ModelProvider {
  fn respond(
    &self,
    request: &ModelRequest,
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
  tool_results: Vec<ModelToolResult>,
}

impl ModelRequest {
  pub fn initial(task: &TaskRequest, model_selection: ModelSelection) -> Self {
    return Self {
      task: task.task().to_owned(),
      model_selection,
      tool_results: Vec::new(),
    };
  }

  pub fn with_tool_output(mut self, tool_call: ModelToolCall, tool_output: ToolOutput) -> Self {
    self.tool_results.push(ModelToolResult {
      tool_call,
      tool_output,
    });

    return self;
  }

  pub fn model_selection(&self) -> &ModelSelection {
    return &self.model_selection;
  }

  pub fn task(&self) -> &str {
    return &self.task;
  }

  pub fn tool_results(&self) -> &[ModelToolResult] {
    return &self.tool_results;
  }
}

#[derive(Debug)]
pub struct ModelToolResult {
  tool_call: ModelToolCall,
  tool_output: ToolOutput,
}

impl ModelToolResult {
  pub fn tool_call(&self) -> &ModelToolCall {
    return &self.tool_call;
  }

  pub fn tool_output(&self) -> &ToolOutput {
    return &self.tool_output;
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

#[cfg(test)]
mod tests {
  use super::{ModelRequest, ModelToolCall};
  use crate::{
    agent::TaskRequest,
    models::{ModelSelection, ReasoningEffort},
    security::{
      ApprovalPolicy, ExecutionPolicy,
      sandbox::{NetworkAccess, SandboxMode},
    },
    tools::{ReadRequest, ToolOutput, ToolRequest},
    workspace::Workspace,
  };
  use std::path::PathBuf;

  fn test_policy() -> ExecutionPolicy {
    return ExecutionPolicy::new(
      SandboxMode::ReadOnly,
      NetworkAccess::Denied,
      ApprovalPolicy::Never,
    )
    .expect("the policy should be valid");
  }

  #[test]
  fn preserves_all_tool_results_for_the_next_provider_request() -> () {
    let task = TaskRequest::new(
      "inspect the project".to_owned(),
      Workspace::new(PathBuf::from("/workspace"), false),
      test_policy(),
    )
    .expect("the task should be valid");
    let selection = ModelSelection::new(
      "codex".to_owned(),
      "gpt-5.6-luna".to_owned(),
      ReasoningEffort::Medium,
    )
    .expect("the selection should be valid");
    let first_call = ModelToolCall::new(
      "call_1".to_owned(),
      ToolRequest::Read(ReadRequest::new(PathBuf::from("Cargo.toml"))),
      r#"{"path":"Cargo.toml"}"#.to_owned(),
    );
    let second_call = ModelToolCall::new(
      "call_2".to_owned(),
      ToolRequest::Read(ReadRequest::new(PathBuf::from("src/main.rs"))),
      r#"{"path":"src/main.rs"}"#.to_owned(),
    );
    let request = ModelRequest::initial(&task, selection)
      .with_tool_output(first_call, ToolOutput::failure())
      .with_tool_output(second_call, ToolOutput::failure());

    assert_eq!(request.tool_results().len(), 2);
    assert_eq!(request.tool_results()[0].tool_call().id(), "call_1");
    assert_eq!(request.tool_results()[1].tool_call().id(), "call_2");

    return;
  }
}
