use std::fmt::{Display, Formatter};

use super::{ProviderKind, codex::CodexAuthError};

pub(crate) enum ProviderError {
  ActiveProvider(ProviderKind),
  ActiveProviderState,
  CodexAuthentication(CodexAuthError),
  UnsupportedIdentifier(String),
}

impl Display for ProviderError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::ActiveProvider(ref provider_kind) => {
        write!(
          formatter,
          "an active provider already exists: {provider_kind}"
        )
      }
      Self::ActiveProviderState => {
        formatter.write_str("could not access the active provider state")
      }
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
