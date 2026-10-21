pub mod auth;

use crate::models::{ModelProvider, ModelProviderError, ModelRequest, ModelResponse};

pub struct CodexProvider {
  access_token: String,
}

impl CodexProvider {
  pub fn new(access_token: String) -> Self {
    return Self { access_token };
  }

  pub fn is_configured(&self) -> bool {
    return !self.access_token.is_empty();
  }
}

impl ModelProvider for CodexProvider {
  fn respond(&self, _request: ModelRequest) -> Result<ModelResponse, ModelProviderError> {
    let is_configured = self.is_configured();

    if !is_configured {
      return Err(ModelProviderError::MissingCredentials);
    }

    return Err(ModelProviderError::NotImplemented);
  }
}
