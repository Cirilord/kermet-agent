use std::fmt::{Display, Formatter};

use super::{ProviderError, codex::CodexProvider};

pub(crate) struct ProviderAccount {
  email: Option<String>,
  provider_kind: ProviderKind,
}

impl ProviderAccount {
  pub(super) fn new(provider_kind: ProviderKind, email: Option<String>) -> ProviderAccount {
    let account = ProviderAccount {
      email,
      provider_kind,
    };

    return account;
  }

  pub(crate) fn email(&self) -> Option<&str> {
    let email = self.email.as_deref();

    return email;
  }

  pub(crate) fn provider_kind(&self) -> &ProviderKind {
    return &self.provider_kind;
  }
}

pub(crate) enum ProviderKind {
  Codex,
}

impl ProviderKind {
  pub(crate) fn parse(identifier: &str) -> Result<ProviderKind, ProviderError> {
    let provider_kind = match identifier {
      "codex" => Self::Codex,
      _ => return Err(ProviderError::UnsupportedIdentifier(identifier.to_owned())),
    };

    return Ok(provider_kind);
  }
}

impl Display for ProviderKind {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::Codex => formatter.write_str("Codex"),
    };

    return result;
  }
}

pub(crate) enum Provider {
  Codex,
}

impl From<ProviderKind> for Provider {
  fn from(provider_kind: ProviderKind) -> Provider {
    let provider = match provider_kind {
      ProviderKind::Codex => Self::Codex,
    };

    return provider;
  }
}

impl Provider {
  pub(crate) fn login(&self) -> Result<ProviderAccount, ProviderError> {
    let result = match *self {
      Self::Codex => CodexProvider.login(),
    };

    return result;
  }
}
