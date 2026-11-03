mod auth;

use super::{ProviderAccount, ProviderError, ProviderKind, ProviderLogout};

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

  pub(super) fn logout(&self) -> Result<ProviderLogout, ProviderError> {
    let result = match auth::logout() {
      Ok(result) => result,
      Err(error) => return Err(ProviderError::CodexAuthentication(error)),
    };
    let provider_logout = match result {
      auth::CodexLogout::NotSignedIn => ProviderLogout::NotSignedIn,
      auth::CodexLogout::RevocationUnconfirmed => ProviderLogout::RevocationUnconfirmed,
      auth::CodexLogout::Revoked => ProviderLogout::Revoked,
    };

    return Ok(provider_logout);
  }
}

pub(super) use auth::CodexAuthError;
