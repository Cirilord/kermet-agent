use super::{ApprovalRequest, AuthorizationDenied};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationError {
  ApprovalRequired(ApprovalRequest),
  Denied(AuthorizationDenied),
}

impl Display for AuthorizationError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::ApprovalRequired(_) => formatter.write_str("approval is required"),
      Self::Denied(ref denied) => denied.fmt(formatter),
    };

    return result;
  }
}

impl std::error::Error for AuthorizationError {}
