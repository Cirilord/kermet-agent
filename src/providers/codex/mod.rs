mod auth;
mod models;

use super::{ProviderAccount, ProviderError, ProviderKind, ProviderLogout, ProviderModel};

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

  pub(super) fn list_models(&self) -> Result<Vec<ProviderModel>, ProviderError> {
    let access_token = match auth::access_token() {
      Ok(access_token) => access_token,
      Err(error) => return Err(ProviderError::CodexAuthentication(error)),
    };
    let access_token = match access_token {
      Some(access_token) => access_token,
      None => {
        return Err(ProviderError::CodexModels(
          models::CodexModelsError::NotSignedIn,
        ));
      }
    };
    let codex_models = match models::list_models(&access_token) {
      Ok(codex_models) => codex_models,
      Err(error) => return Err(ProviderError::CodexModels(error)),
    };
    let mut provider_models = Vec::new();

    for model in codex_models {
      let provider_model = ProviderModel::new(
        model.slug().to_owned(),
        model.display_name().to_owned(),
        model.supported_reasoning_levels().to_vec(),
        model.default_reasoning_level().to_owned(),
        model.context_window(),
        model.max_context_window(),
      );

      provider_models.push(provider_model);
    }

    return Ok(provider_models);
  }
}

pub(super) use auth::CodexAuthError;
pub(super) use models::CodexModelsError;
