use std::fmt::{Display, Formatter};

use super::{ProviderError, codex::CodexProvider, state::ActiveProviderStore};

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

  pub(super) fn identifier(&self) -> &str {
    let identifier = match *self {
      Self::Codex => "codex",
    };

    return identifier;
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
    let active_provider = match ActiveProviderStore::load() {
      Ok(active_provider) => active_provider,
      Err(error) => return Err(error),
    };
    if let Some(provider_kind) = active_provider {
      return Err(ProviderError::ActiveProvider(provider_kind));
    }

    let result = match *self {
      Self::Codex => CodexProvider.login(),
    };
    let account = match result {
      Ok(account) => account,
      Err(error) => return Err(error),
    };
    let provider_kind = self.kind();

    if let Err(error) = ActiveProviderStore::save(&provider_kind) {
      return Err(error);
    }

    return Ok(account);
  }

  fn kind(&self) -> ProviderKind {
    let provider_kind = match *self {
      Self::Codex => ProviderKind::Codex,
    };

    return provider_kind;
  }
}
