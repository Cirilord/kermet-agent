use std::fmt::{Display, Formatter};

use super::codex::CodexAuthError;

pub(crate) enum ProviderError {
  CodexAuthentication(CodexAuthError),
  UnsupportedIdentifier(String),
}

impl Display for ProviderError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::CodexAuthentication(ref error) => {
        write!(formatter, "Codex authentication failed: {error}")
      }
      Self::UnsupportedIdentifier(ref identifier) => {
        write!(formatter, "unsupported provider: {identifier}")
      }
    };

    return result;
  }
}
