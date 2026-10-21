use crate::workspace::Workspace;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionMode {
  ReadOnly,
  WorkspaceWrite,
}

impl Display for PermissionMode {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let value = match *self {
      Self::ReadOnly => "read-only",
      Self::WorkspaceWrite => "workspace-write",
    };

    return formatter.write_str(value);
  }
}

#[derive(Debug)]
pub struct TaskRequest {
  task: String,
  workspace: Workspace,
  permission_mode: PermissionMode,
}

impl TaskRequest {
  pub fn new(
    task: String,
    workspace: Workspace,
    permission_mode: PermissionMode,
  ) -> Result<Self, TaskRequestError> {
    let task = task.trim();

    if task.is_empty() {
      return Err(TaskRequestError::EmptyTask);
    }

    return Ok(Self {
      task: task.to_owned(),
      workspace,
      permission_mode,
    });
  }

  pub fn permission_mode(&self) -> PermissionMode {
    return self.permission_mode;
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
  use super::{PermissionMode, TaskRequest, TaskRequestError};
  use crate::workspace::Workspace;
  use std::path::PathBuf;

  #[test]
  fn creates_a_request_with_a_normalized_task() -> () {
    let workspace = Workspace::new(PathBuf::from("/workspace"), false);
    let request = TaskRequest::new(
      "  inspect the project  ".to_owned(),
      workspace,
      PermissionMode::ReadOnly,
    )
    .expect("the request should be valid");

    assert_eq!(request.task(), "inspect the project");
    assert_eq!(request.workspace().path(), PathBuf::from("/workspace"));
    assert_eq!(request.permission_mode(), PermissionMode::ReadOnly);

    return;
  }

  #[test]
  fn rejects_an_empty_task() -> () {
    let workspace = Workspace::new(PathBuf::from("/workspace"), false);
    let result = TaskRequest::new("  ".to_owned(), workspace, PermissionMode::ReadOnly);

    assert_eq!(result.unwrap_err(), TaskRequestError::EmptyTask);

    return;
  }
}
