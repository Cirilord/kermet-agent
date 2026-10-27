const MAX_COMMAND_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug)]
pub struct BashRequest {
  command: String,
}

impl BashRequest {
  pub fn new(command: String) -> Result<Self, BashRequestError> {
    let command = command.trim();

    if command.is_empty() {
      return Err(BashRequestError::EmptyCommand);
    }
    if command.len() > MAX_COMMAND_BYTES {
      return Err(BashRequestError::TooLong {
        limit: MAX_COMMAND_BYTES,
      });
    }

    return Ok(Self {
      command: command.to_owned(),
    });
  }

  pub fn command(&self) -> &str {
    return &self.command;
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BashRequestError {
  EmptyCommand,
  TooLong { limit: usize },
}
