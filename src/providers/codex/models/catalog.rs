use std::time::Duration;

use reqwest::{blocking::Client as HttpClient, redirect::Policy};
use serde::Deserialize;

use super::CodexModelsError;

const MODELS_ENDPOINT: &str = "https://api.openai.com/v1/models";

pub(in super::super) struct CodexModel {
  context_window: u64,
  default_reasoning_level: String,
  display_name: String,
  max_context_window: u64,
  slug: String,
  supported_reasoning_levels: Vec<String>,
}

impl CodexModel {
  pub(in super::super) fn context_window(&self) -> u64 {
    return self.context_window;
  }

  pub(in super::super) fn default_reasoning_level(&self) -> &str {
    return &self.default_reasoning_level;
  }

  pub(in super::super) fn display_name(&self) -> &str {
    return &self.display_name;
  }

  pub(in super::super) fn max_context_window(&self) -> u64 {
    return self.max_context_window;
  }

  pub(in super::super) fn slug(&self) -> &str {
    return &self.slug;
  }

  pub(in super::super) fn supported_reasoning_levels(&self) -> &[String] {
    return &self.supported_reasoning_levels;
  }
}

#[derive(Deserialize)]
pub(super) struct ModelCatalogResponse {
  pub(super) models: Vec<ModelResponse>,
}

#[derive(Deserialize)]
pub(super) struct ModelResponse {
  pub(super) context_window: u64,
  pub(super) default_reasoning_level: String,
  pub(super) display_name: String,
  pub(super) max_context_window: u64,
  pub(super) slug: String,
  pub(super) supported_reasoning_levels: Vec<ReasoningLevelResponse>,
  pub(super) visibility: String,
}

#[derive(Deserialize)]
pub(super) struct ReasoningLevelResponse {
  pub(super) effort: String,
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
      context_window: model.context_window,
      default_reasoning_level: model.default_reasoning_level,
      display_name: model.display_name,
      max_context_window: model.max_context_window,
      slug: model.slug,
      supported_reasoning_levels: model
        .supported_reasoning_levels
        .into_iter()
        .map(|level| {
          return level.effort;
        })
        .collect(),
    };

    models.push(model);
  }

  return models;
}
