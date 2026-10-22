use crate::{
  models::{
    ModelEvent, ModelProvider, ModelProviderError, ModelRequest, ModelResponse, ModelSelection,
  },
  providers::{CodexProvider, codex::auth::CodexCredentialStore},
  security::CredentialStore,
};

pub struct ProviderRegistry<S> {
  credentials: S,
}

impl<S> ProviderRegistry<S>
where
  S: CredentialStore,
{
  pub fn new(credentials: S) -> Self {
    return Self { credentials };
  }

  pub fn resolve(
    &self,
    selection: &ModelSelection,
    profile: &str,
  ) -> Result<ConfiguredModelProvider, ProviderRegistryError> {
    let provider = match selection.provider() {
      "codex" => self.resolve_codex(profile),
      _ => return Err(ProviderRegistryError::UnsupportedProvider),
    };

    return provider;
  }

  fn resolve_codex(&self, profile: &str) -> Result<ConfiguredModelProvider, ProviderRegistryError> {
    let credentials = match CodexCredentialStore::new(&self.credentials, profile.to_owned()) {
      Ok(credentials) => credentials,
      Err(_) => return Err(ProviderRegistryError::CredentialStoreUnavailable),
    };
    let tokens = match credentials.load() {
      Ok(Some(tokens)) => tokens,
      Ok(None) => return Err(ProviderRegistryError::MissingCredentials),
      Err(_) => return Err(ProviderRegistryError::CredentialStoreUnavailable),
    };
    let provider =
      ConfiguredModelProvider::Codex(CodexProvider::new(tokens.access_token().to_owned()));

    return Ok(provider);
  }
}

pub enum ConfiguredModelProvider {
  Codex(CodexProvider),
}

impl ModelProvider for ConfiguredModelProvider {
  fn respond(
    &self,
    request: ModelRequest,
    on_event: &mut dyn FnMut(ModelEvent),
  ) -> Result<ModelResponse, ModelProviderError> {
    let response = match *self {
      Self::Codex(ref provider) => provider.respond(request, on_event),
    };

    return response;
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ProviderRegistryError {
  UnsupportedProvider,
  MissingCredentials,
  CredentialStoreUnavailable,
}

#[cfg(test)]
mod tests {
  use super::{ConfiguredModelProvider, ProviderRegistry, ProviderRegistryError};
  use crate::{
    models::{ModelSelection, ReasoningEffort},
    providers::codex::auth::{CodexCredentialStore, CodexTokens},
    security::EphemeralCredentialStore,
  };
  use std::time::{Duration, SystemTime};

  fn selection(provider: &str) -> ModelSelection {
    let selection = ModelSelection::new(
      provider.to_owned(),
      "gpt-6.1-sol".to_owned(),
      ReasoningEffort::Medium,
    )
    .expect("the selection should be valid");

    return selection;
  }

  #[test]
  fn rejects_an_unsupported_provider() -> () {
    let credentials = EphemeralCredentialStore::new();
    let registry = ProviderRegistry::new(&credentials);
    let result = registry.resolve(&selection("unknown"), "default");
    let error = match result {
      Err(error) => error,
      Ok(_) => panic!("the unsupported provider should be rejected"),
    };

    assert_eq!(error, ProviderRegistryError::UnsupportedProvider);

    return;
  }

  #[test]
  fn rejects_codex_without_credentials() -> () {
    let credentials = EphemeralCredentialStore::new();
    let registry = ProviderRegistry::new(&credentials);
    let result = registry.resolve(&selection("codex"), "default");
    let error = match result {
      Err(error) => error,
      Ok(_) => panic!("Codex without credentials should be rejected"),
    };

    assert_eq!(error, ProviderRegistryError::MissingCredentials);

    return;
  }

  #[test]
  fn creates_a_configured_codex_provider() -> () {
    let credentials = EphemeralCredentialStore::new();
    let store = CodexCredentialStore::new(&credentials, "default".to_owned())
      .expect("the credential store should be created");
    let tokens = CodexTokens::new(
      "access-token".to_owned(),
      "refresh-token".to_owned(),
      SystemTime::UNIX_EPOCH + Duration::from_secs(3600),
    );

    store.save(&tokens).expect("the tokens should be saved");

    let registry = ProviderRegistry::new(&credentials);
    let provider = registry
      .resolve(&selection("codex"), "default")
      .expect("the Codex provider should be created");

    assert!(matches!(provider, ConfiguredModelProvider::Codex(_)));

    return;
  }
}
