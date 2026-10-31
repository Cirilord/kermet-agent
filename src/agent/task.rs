use crate::{security::ExecutionPolicy, workspace::Workspace};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct TaskRequest {
  task: String,
  workspace: Workspace,
  execution_policy: ExecutionPolicy,
}

impl TaskRequest {
  pub fn new(
    task: String,
    workspace: Workspace,
    execution_policy: ExecutionPolicy,
  ) -> Result<Self, TaskRequestError> {
    let task = task.trim();

    if task.is_empty() {
      return Err(TaskRequestError::EmptyTask);
    }

    return Ok(Self {
      task: task.to_owned(),
      workspace,
      execution_policy,
    });
  }

  pub fn execution_policy(&self) -> ExecutionPolicy {
    return self.execution_policy;
  }

  pub fn task(&self) -> &str {
    return &self.task;
  }

  pub fn workspace(&self) -> &Workspace {
    return &self.workspace;
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TaskRequestError {
  EmptyTask,
}

impl Display for TaskRequestError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::EmptyTask => formatter.write_str("task must not be empty"),
    };

    return result;
  }
}

impl std::error::Error for TaskRequestError {}

#[cfg(test)]
mod tests {
  use super::{TaskRequest, TaskRequestError};
  use crate::{
    security::{
      ApprovalPolicy, ExecutionPolicy,
      sandbox::{NetworkAccess, SandboxMode},
    },
    workspace::Workspace,
  };
  use std::path::PathBuf;

  #[test]
  fn creates_a_request_with_a_normalized_task() -> () {
    let workspace = Workspace::new(PathBuf::from("/workspace"), false);
    let request = TaskRequest::new("  inspect the project  ".to_owned(), workspace, policy())
      .expect("the request should be valid");

    assert_eq!(request.task(), "inspect the project");
    assert_eq!(request.workspace().path(), PathBuf::from("/workspace"));
    assert_eq!(request.execution_policy(), policy());

    return;
  }

  #[test]
  fn rejects_an_empty_task() -> () {
    let workspace = Workspace::new(PathBuf::from("/workspace"), false);
    let result = TaskRequest::new("  ".to_owned(), workspace, policy());

    assert_eq!(result.unwrap_err(), TaskRequestError::EmptyTask);

    return;
  }

  fn policy() -> ExecutionPolicy {
    return ExecutionPolicy::new(
      SandboxMode::ReadOnly,
      NetworkAccess::Denied,
      ApprovalPolicy::OnRequest,
    )
    .expect("the policy should be valid");
  }
}
