use super::super::ExecutionPolicy;
use super::ExecutionRequirement;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizationDenied {
  policy: ExecutionPolicy,
  requirement: ExecutionRequirement,
}

impl AuthorizationDenied {
  pub(super) fn new(policy: ExecutionPolicy, requirement: ExecutionRequirement) -> Self {
    return Self {
      policy,
      requirement,
    };
  }

  pub fn policy(&self) -> ExecutionPolicy {
    return self.policy;
  }

  pub fn requirement(&self) -> ExecutionRequirement {
    return self.requirement;
  }
}

impl Display for AuthorizationDenied {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = formatter.write_str("the execution requirement is denied by the active policy");

    return result;
  }
}

impl std::error::Error for AuthorizationDenied {}
