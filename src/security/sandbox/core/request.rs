use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxRequest {
  program: String,
  arguments: Vec<String>,
  environment: BTreeMap<String, String>,
}

impl SandboxRequest {
  pub fn new(program: String, arguments: Vec<String>) -> Result<Self, SandboxRequestError> {
    let program = program.trim();
    if program.is_empty() {
      return Err(SandboxRequestError::EmptyProgram);
    }

    return Ok(Self {
      program: program.to_owned(),
      arguments,
      environment: BTreeMap::new(),
    });
  }

  pub fn with_environment(
    mut self,
    name: String,
    value: String,
  ) -> Result<Self, SandboxRequestError> {
    if !is_valid_environment_name(&name) {
      return Err(SandboxRequestError::InvalidEnvironmentName);
    }

    self.environment.insert(name, value);

    return Ok(self);
  }

  pub fn program(&self) -> &str {
    return &self.program;
  }

  pub fn arguments(&self) -> &[String] {
    return &self.arguments;
  }

  pub fn environment(&self) -> &BTreeMap<String, String> {
    return &self.environment;
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SandboxRequestError {
  EmptyProgram,
  InvalidEnvironmentName,
}

fn is_valid_environment_name(name: &str) -> bool {
  let mut characters = name.chars();
  let first = match characters.next() {
    Some(character) => character,
    None => return false,
  };
  if !first.is_ascii_alphabetic() && first != '_' {
    return false;
  }

  let valid =
    characters.all(|character| return character.is_ascii_alphanumeric() || character == '_');

  return valid;
}
