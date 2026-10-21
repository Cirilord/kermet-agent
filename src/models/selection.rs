use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelSelection {
  provider: String,
  model: String,
  reasoning_effort: ReasoningEffort,
}

impl ModelSelection {
  pub fn new(
    provider: String,
    model: String,
    reasoning_effort: ReasoningEffort,
  ) -> Result<Self, ModelSelectionError> {
    if provider.trim().is_empty() {
      return Err(ModelSelectionError::MissingProvider);
    }
    if model.trim().is_empty() {
      return Err(ModelSelectionError::MissingModel);
    }

    return Ok(Self {
      provider,
      model,
      reasoning_effort,
    });
  }

  pub fn model(&self) -> &str {
    return &self.model;
  }

  pub fn provider(&self) -> &str {
    return &self.provider;
  }

  pub fn reasoning_effort(&self) -> ReasoningEffort {
    return self.reasoning_effort;
  }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ReasoningEffort {
  None,
  Minimal,
  Low,
  Medium,
  High,
  XHigh,
  Max,
}

impl ReasoningEffort {
  pub fn from_identifier(value: &str) -> Option<Self> {
    let reasoning_effort = match value {
      "none" => Self::None,
      "minimal" => Self::Minimal,
      "low" => Self::Low,
      "medium" => Self::Medium,
      "high" => Self::High,
      "xhigh" => Self::XHigh,
      "max" => Self::Max,
      _ => return None,
    };

    return Some(reasoning_effort);
  }

  pub fn identifier(&self) -> &'static str {
    let identifier = match *self {
      Self::None => "none",
      Self::Minimal => "minimal",
      Self::Low => "low",
      Self::Medium => "medium",
      Self::High => "high",
      Self::XHigh => "xhigh",
      Self::Max => "max",
    };

    return identifier;
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ModelSelectionError {
  MissingProvider,
  MissingModel,
}
