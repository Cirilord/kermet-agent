use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq)]
pub enum ExecutionPolicyError {
  DangerFullAccessRequiresNetwork,
}

impl Display for ExecutionPolicyError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::DangerFullAccessRequiresNetwork => {
        formatter.write_str("danger-full-access requires network access")
      }
    };

    return result;
  }
}

impl std::error::Error for ExecutionPolicyError {}
