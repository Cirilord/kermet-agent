use std::fmt::{Display, Formatter};

use super::{
  ProviderKind,
  codex::{CodexAuthError, CodexModelsError},
};

pub(crate) enum ProviderError {
  ActiveProvider(ProviderKind),
  ActiveProviderState,
  CodexAuthentication(CodexAuthError),
  CodexModels(CodexModelsError),
  ModelNotFound(String),
  NoActiveProvider,
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
      Self::CodexModels(ref error) => write!(formatter, "Codex models failed: {error}"),
      Self::ModelNotFound(ref slug) => write!(formatter, "model not found: {slug}"),
      Self::NoActiveProvider => formatter.write_str("no active provider is configured"),
      Self::UnsupportedIdentifier(ref identifier) => {
        write!(formatter, "unsupported provider: {identifier}")
      }
    };

    return result;
  }
}
