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
  pub fn identifier(&self) -> &'static str {
    let identifier = match *self {
      Self::ReadFile => "read-file",
    };

    return identifier;
  }
}

#[derive(Debug)]
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
}

#[derive(Debug)]
pub enum ToolOutput {
  ReadFile(ReadFileOutput),
}

#[derive(Debug)]
pub enum ToolExecutionError {
  ReadFile(ReadFileError),
}
