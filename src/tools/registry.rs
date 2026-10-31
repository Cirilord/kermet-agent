use crate::{
  agent::TaskRequest,
  security::AuthorizationManager,
  tools::{
    Bash, BashError, BashOutput, BashRequest, Edit, EditError, EditOutput, EditRequest, Read,
    ReadError, ReadOutput, ReadRequest, Write, WriteError, WriteOutput, WriteRequest,
  },
};

pub struct ToolRegistry {
  bash: Bash,
  edit: Edit,
  read: Read,
  write: Write,
}

impl ToolRegistry {
  pub fn new(authorization_manager: AuthorizationManager) -> Self {
    return Self {
      bash: Bash::new(authorization_manager.clone()),
      edit: Edit::new(authorization_manager.clone()),
      read: Read::new(authorization_manager.clone()),
      write: Write::new(authorization_manager),
    };
  }

  pub fn execute(
    &self,
    task: &TaskRequest,
    request: ToolRequest,
  ) -> Result<ToolOutput, ToolExecutionError> {
    let output = match request {
      ToolRequest::Bash(request) => match self.bash.execute(task, request) {
        Ok(output) => ToolOutput::Bash(output),
        Err(error) => return Err(ToolExecutionError::Bash(error)),
      },
      ToolRequest::Edit(request) => match self.edit.execute(task, request) {
        Ok(output) => ToolOutput::Edit(output),
        Err(error) => return Err(ToolExecutionError::Edit(error)),
      },
      ToolRequest::Read(request) => match self.read.execute(task, request) {
        Ok(output) => ToolOutput::Read(output),
        Err(error) => return Err(ToolExecutionError::Read(error)),
      },
      ToolRequest::Write(request) => match self.write.execute(task, request) {
        Ok(output) => ToolOutput::Write(output),
        Err(error) => return Err(ToolExecutionError::Write(error)),
      },
    };

    return Ok(output);
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolName {
  Bash,
  Edit,
  Read,
  Write,
}

impl ToolName {
  pub fn function_name(&self) -> &'static str {
    let function_name = match *self {
      Self::Bash => "bash",
      Self::Edit => "edit",
      Self::Read => "read",
      Self::Write => "write",
    };

    return function_name;
  }

  pub fn identifier(&self) -> &'static str {
    let identifier = match *self {
      Self::Bash => "bash",
      Self::Edit => "edit",
      Self::Read => "read",
      Self::Write => "write",
    };

    return identifier;
  }
}

#[derive(Clone, Debug)]
pub enum ToolRequest {
  Bash(BashRequest),
  Edit(EditRequest),
  Read(ReadRequest),
  Write(WriteRequest),
}

impl ToolRequest {
  pub fn name(&self) -> ToolName {
    let name = match *self {
      Self::Bash(_) => ToolName::Bash,
      Self::Edit(_) => ToolName::Edit,
      Self::Read(_) => ToolName::Read,
      Self::Write(_) => ToolName::Write,
    };

    return name;
  }

  pub fn target(&self) -> String {
    let target = match *self {
      Self::Bash(ref request) => request.command().to_owned(),
      Self::Edit(ref request) => request.path().display().to_string(),
      Self::Read(ref request) => request.path().display().to_string(),
      Self::Write(ref request) => request.path().display().to_string(),
    };

    return target;
  }
}

impl ToolOutput {
  pub fn failure() -> Self {
    return Self::Failure;
  }

  pub fn model_content(&self) -> String {
    let content = match *self {
      Self::Bash(ref output) => {
        let stdout = String::from_utf8_lossy(output.stdout());
        let stderr = String::from_utf8_lossy(output.stderr());
        serde_json::json!({
          "exit_code": output.exit_code(),
          "timed_out": output.timed_out(),
          "stdout": stdout,
          "stderr": stderr,
          "stdout_truncated": output.stdout_truncated(),
          "stderr_truncated": output.stderr_truncated(),
        })
        .to_string()
      }
      Self::Edit(ref output) => serde_json::json!({
        "path": output.path().display().to_string(),
        "bytes_written": output.bytes_written(),
      })
      .to_string(),
      Self::Read(ref output) => {
        let contents = String::from_utf8_lossy(output.contents());
        serde_json::json!({
          "path": output.path().display().to_string(),
          "content": contents,
        })
        .to_string()
      }
      Self::Write(ref output) => serde_json::json!({
        "path": output.path().display().to_string(),
        "bytes_written": output.bytes_written(),
      })
      .to_string(),
      Self::Failure => serde_json::json!({
        "error": "The requested tool operation could not be completed.",
      })
      .to_string(),
    };

    return content;
  }
}

#[derive(Debug)]
pub enum ToolOutput {
  Bash(BashOutput),
  Edit(EditOutput),
  Read(ReadOutput),
  Write(WriteOutput),
  Failure,
}

#[derive(Debug)]
pub enum ToolExecutionError {
  Bash(BashError),
  Edit(EditError),
  Read(ReadError),
  Write(WriteError),
}

#[cfg(test)]
mod tests {
  use super::{BashOutput, ToolName, ToolOutput};

  #[test]
  fn exposes_a_stable_read_identifier() -> () {
    assert_eq!(ToolName::Read.identifier(), "read");

    return;
  }

  #[test]
  fn exposes_a_stable_write_identifier() -> () {
    assert_eq!(ToolName::Write.identifier(), "write");

    return;
  }

  #[test]
  fn exposes_a_stable_edit_identifier() -> () {
    assert_eq!(ToolName::Edit.identifier(), "edit");

    return;
  }

  #[test]
  fn renders_a_safe_failure_for_the_model() -> () {
    let output = ToolOutput::failure();

    assert_eq!(
      output.model_content(),
      r#"{"error":"The requested tool operation could not be completed."}"#
    );

    return;
  }

  #[test]
  fn renders_bash_output_for_the_model() -> () {
    let output = ToolOutput::Bash(BashOutput::new(
      Some(0),
      false,
      b"passed".to_vec(),
      Vec::new(),
      false,
      false,
    ));

    assert_eq!(
      output.model_content(),
      r#"{"exit_code":0,"stderr":"","stderr_truncated":false,"stdout":"passed","stdout_truncated":false,"timed_out":false}"#
    );

    return;
  }
}
