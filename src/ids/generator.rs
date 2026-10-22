const TOKEN_LENGTH: usize = 16;

#[derive(Debug, Eq, PartialEq)]
pub enum IdError {
  InvalidPrefix,
  InvalidValue,
}

fn is_valid_prefix(prefix: &str) -> bool {
  let mut characters = prefix.bytes();
  let first = match characters.next() {
    Some(first) => first,
    None => return false,
  };
  if !first.is_ascii_lowercase() {
    return false;
  }
  let valid = characters.all(|character| {
    return character.is_ascii_lowercase() || character.is_ascii_digit() || character == b'-';
  });

  return valid;
}

pub struct IdGenerator;

impl IdGenerator {
  pub fn generate(prefix: &str) -> Result<String, IdError> {
    if !is_valid_prefix(prefix) {
      return Err(IdError::InvalidPrefix);
    }
    let value = format!("{prefix}-{:016x}", rand::random::<u64>());

    return Ok(value);
  }

  pub fn validate(prefix: &str, value: &str) -> Result<(), IdError> {
    if !is_valid_prefix(prefix) {
      return Err(IdError::InvalidPrefix);
    }
    let token = match value.strip_prefix(&format!("{prefix}-")) {
      Some(token) => token,
      None => return Err(IdError::InvalidValue),
    };
    if token.len() != TOKEN_LENGTH || !token.bytes().all(|byte| return byte.is_ascii_hexdigit()) {
      return Err(IdError::InvalidValue);
    }

    return Ok(());
  }
}
