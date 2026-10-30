use super::ExecutionRequirement;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovalRequest {
  requirement: ExecutionRequirement,
}

impl ApprovalRequest {
  pub(super) fn new(requirement: ExecutionRequirement) -> Self {
    return Self { requirement };
  }

  pub fn requirement(&self) -> ExecutionRequirement {
    return self.requirement;
  }
}
