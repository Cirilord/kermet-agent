use std::fmt::{Display, Formatter};

use super::{ProviderError, codex::CodexProvider, state::ActiveProviderStore};

pub(crate) struct ProviderAccount {
  email: Option<String>,
  provider_kind: ProviderKind,
}

pub(crate) struct ProviderModel {
  context_window: u64,
  default_reasoning_level: String,
  display_name: String,
  max_context_window: u64,
  slug: String,
  supported_reasoning_levels: Vec<String>,
}

pub(crate) enum ProviderLogout {
  NotSignedIn,
  RevocationUnconfirmed,
  Revoked,
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

impl ProviderModel {
  pub(super) fn new(
    slug: String,
    display_name: String,
    supported_reasoning_levels: Vec<String>,
    default_reasoning_level: String,
    context_window: u64,
    max_context_window: u64,
  ) -> ProviderModel {
    let model = ProviderModel {
      context_window,
      default_reasoning_level,
      display_name,
      max_context_window,
      slug,
      supported_reasoning_levels,
    };

    return model;
  }

  pub(crate) fn context_window(&self) -> u64 {
    return self.context_window;
  }

  pub(crate) fn default_reasoning_level(&self) -> &str {
    return &self.default_reasoning_level;
  }

  pub(crate) fn display_name(&self) -> &str {
    return &self.display_name;
  }

  pub(crate) fn max_context_window(&self) -> u64 {
    return self.max_context_window;
  }

  pub(crate) fn slug(&self) -> &str {
    return &self.slug;
  }

  pub(crate) fn supported_reasoning_levels(&self) -> &[String] {
    return &self.supported_reasoning_levels;
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

  pub(crate) fn logout() -> Result<ProviderLogout, ProviderError> {
    let active_provider = match ActiveProviderStore::load() {
      Ok(active_provider) => active_provider,
      Err(error) => return Err(error),
    };
    let provider_kind = match active_provider {
      Some(provider_kind) => provider_kind,
      None => return Ok(ProviderLogout::NotSignedIn),
    };
    let result = match provider_kind {
      ProviderKind::Codex => CodexProvider.logout(),
    };
    let provider_logout = match result {
      Ok(result) => result,
      Err(error) => return Err(error),
    };

    if let Err(error) = ActiveProviderStore::clear() {
      return Err(error);
    }

    return Ok(provider_logout);
  }

  pub(crate) fn list_models() -> Result<Vec<ProviderModel>, ProviderError> {
    let active_provider = match ActiveProviderStore::load() {
      Ok(active_provider) => active_provider,
      Err(error) => return Err(error),
    };
    let result = match active_provider {
      Some(ProviderKind::Codex) => CodexProvider.list_models(),
      None => return Err(ProviderError::NoActiveProvider),
    };

    return result;
  }

  fn kind(&self) -> ProviderKind {
    let provider_kind = match *self {
      Self::Codex => ProviderKind::Codex,
    };

    return provider_kind;
  }
}
