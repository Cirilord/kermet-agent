use std::time::Duration;

use reqwest::{blocking::Client as HttpClient, redirect::Policy};
use serde::Deserialize;

use super::CodexModelsError;

const MODELS_ENDPOINT: &str = "https://api.openai.com/v1/models";

pub(in super::super) struct CodexModel {
  display_name: String,
  slug: String,
}

impl CodexModel {
  pub(in super::super) fn display_name(&self) -> &str {
    return &self.display_name;
  }

  pub(in super::super) fn slug(&self) -> &str {
    return &self.slug;
  }
}

#[derive(Deserialize)]
pub(super) struct ModelCatalogResponse {
  pub(super) models: Vec<ModelResponse>,
}

#[derive(Deserialize)]
pub(super) struct ModelResponse {
  pub(super) display_name: String,
  pub(super) slug: String,
  pub(super) visibility: String,
}

pub(in super::super) fn list_models(
  access_token: &str,
) -> Result<Vec<CodexModel>, CodexModelsError> {
  let client = match HttpClient::builder()
    .redirect(Policy::none())
    .timeout(Duration::from_secs(15))
    .build()
  {
    Ok(client) => client,
    Err(_) => return Err(CodexModelsError::Request),
  };
  let response = match client.get(MODELS_ENDPOINT).bearer_auth(access_token).send() {
    Ok(response) => response,
    Err(_) => return Err(CodexModelsError::Request),
  };

  if !response.status().is_success() {
    return Err(CodexModelsError::Request);
  }
  let response = match response.json::<ModelCatalogResponse>() {
    Ok(response) => response,
    Err(_) => return Err(CodexModelsError::Request),
  };
  let models = models_from_response(response);

  return Ok(models);
}

pub(super) fn models_from_response(response: ModelCatalogResponse) -> Vec<CodexModel> {
  let mut models = Vec::new();

  for model in response.models {
    if model.visibility != "list" {
      continue;
    }
    let model = CodexModel {
      display_name: model.display_name,
      slug: model.slug,
    };

    models.push(model);
  }

  return models;
}
