mod auth;

use super::{ProviderAccount, ProviderError, ProviderKind};

pub(super) struct CodexProvider;

impl CodexProvider {
  pub(super) fn login(&self) -> Result<ProviderAccount, ProviderError> {
    let account = match auth::login() {
      Ok(account) => account,
      Err(error) => return Err(ProviderError::CodexAuthentication(error)),
    };
    let provider_account = ProviderAccount::new(ProviderKind::Codex, account.into_email());

    return Ok(provider_account);
  }
}

pub(super) use auth::CodexAuthError;
