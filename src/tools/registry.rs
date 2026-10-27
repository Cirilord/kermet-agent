use crate::{
  agent::TaskRequest,
  security::PermissionManager,
  tools::{
    Bash, BashError, BashOutput, BashRequest, ReadFile, ReadFileError, ReadFileOutput,
    ReadFileRequest, Write, WriteError, WriteOutput, WriteRequest,
  },
};

pub struct ToolRegistry {
  bash: Bash,
  read_file: ReadFile,
  write: Write,
}

impl ToolRegistry {
  pub fn new(permission_manager: PermissionManager) -> Self {
    return Self {
      bash: Bash::new(permission_manager.clone()),
      read_file: ReadFile::new(permission_manager.clone()),
      write: Write::new(permission_manager),
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
      ToolRequest::ReadFile(request) => match self.read_file.execute(task, request) {
        Ok(output) => ToolOutput::ReadFile(output),
        Err(error) => return Err(ToolExecutionError::ReadFile(error)),
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
  ReadFile,
  Write,
}

impl ToolName {
  pub fn function_name(&self) -> &'static str {
    let function_name = match *self {
      Self::Bash => "bash",
      Self::ReadFile => "read_file",
      Self::Write => "write",
    };

    return function_name;
  }

  pub fn identifier(&self) -> &'static str {
    let identifier = match *self {
      Self::Bash => "bash",
      Self::ReadFile => "read-file",
      Self::Write => "write-file",
    };

    return identifier;
  }
}

#[derive(Clone, Debug)]
pub enum ToolRequest {
  Bash(BashRequest),
  ReadFile(ReadFileRequest),
  Write(WriteRequest),
}

impl ToolRequest {
  pub fn name(&self) -> ToolName {
    let name = match *self {
      Self::Bash(_) => ToolName::Bash,
      Self::ReadFile(_) => ToolName::ReadFile,
      Self::Write(_) => ToolName::Write,
    };

    return name;
  }

  pub fn target(&self) -> String {
    let target = match *self {
      Self::Bash(ref request) => request.command().to_owned(),
      Self::ReadFile(ref request) => request.path().display().to_string(),
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
      Self::ReadFile(ref output) => {
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
  ReadFile(ReadFileOutput),
  Write(WriteOutput),
  Failure,
}

#[derive(Debug)]
pub enum ToolExecutionError {
  Bash(BashError),
  ReadFile(ReadFileError),
  Write(WriteError),
}

#[cfg(test)]
mod tests {
  use super::{BashOutput, ToolOutput};

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
