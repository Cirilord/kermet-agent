use crate::{
  agent::TaskRequest,
  security::PermissionManager,
  tools::{ReadFile, ReadFileError, ReadFileOutput, ReadFileRequest},
};

pub struct ToolRegistry {
  read_file: ReadFile,
}

impl ToolRegistry {
  pub fn new(permission_manager: PermissionManager) -> Self {
    return Self {
      read_file: ReadFile::new(permission_manager),
    };
  }

  pub fn execute(
    &self,
    task: &TaskRequest,
    request: ToolRequest,
  ) -> Result<ToolOutput, ToolExecutionError> {
    let output = match request {
      ToolRequest::ReadFile(request) => match self.read_file.execute(task, request) {
        Ok(output) => ToolOutput::ReadFile(output),
        Err(error) => return Err(ToolExecutionError::ReadFile(error)),
      },
    };

    return Ok(output);
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolName {
  ReadFile,
}

impl ToolName {
  pub fn function_name(&self) -> &'static str {
    let function_name = match *self {
      Self::ReadFile => "read_file",
    };

    return function_name;
  }

  pub fn identifier(&self) -> &'static str {
    let identifier = match *self {
      Self::ReadFile => "read-file",
    };

    return identifier;
  }
}

#[derive(Clone, Debug)]
pub enum ToolRequest {
  ReadFile(ReadFileRequest),
}

impl ToolRequest {
  pub fn name(&self) -> ToolName {
    let name = match *self {
      Self::ReadFile(_) => ToolName::ReadFile,
    };

    return name;
  }

  pub fn target(&self) -> String {
    let target = match *self {
      Self::ReadFile(ref request) => request.path().display().to_string(),
    };

    return target;
  }
}

impl ToolOutput {
  pub fn model_content(&self) -> String {
    let content = match *self {
      Self::ReadFile(ref output) => {
        let contents = String::from_utf8_lossy(output.contents());
        serde_json::json!({
          "path": output.path().display().to_string(),
          "content": contents,
        })
        .to_string()
      }
    };

    return content;
  }
}

#[derive(Debug)]
pub enum ToolOutput {
  ReadFile(ReadFileOutput),
}

#[derive(Debug)]
pub enum ToolExecutionError {
  ReadFile(ReadFileError),
}
