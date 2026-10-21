use crate::models::{ModelProvider, ModelProviderError, ModelRequest, ModelResponse};
use std::{collections::VecDeque, sync::Mutex};

pub struct FakeModelProvider {
  responses: Mutex<VecDeque<ModelResponse>>,
}

impl FakeModelProvider {
  pub fn new(responses: Vec<ModelResponse>) -> Self {
    return Self {
      responses: Mutex::new(VecDeque::from(responses)),
    };
  }
}

impl ModelProvider for FakeModelProvider {
  fn respond(&self, _request: ModelRequest) -> Result<ModelResponse, ModelProviderError> {
    let mut responses = match self.responses.lock() {
      Ok(responses) => responses,
      Err(_) => return Err(ModelProviderError::Exhausted),
    };
    let response = match responses.pop_front() {
      Some(response) => response,
      None => return Err(ModelProviderError::Exhausted),
    };

    return Ok(response);
  }
}
