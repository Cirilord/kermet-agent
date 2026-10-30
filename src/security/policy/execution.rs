use super::super::sandbox::{NetworkAccess, SandboxMode};
use super::{ApprovalPolicy, ExecutionPolicyError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionPolicy {
  sandbox_mode: SandboxMode,
  network_access: NetworkAccess,
  approval_policy: ApprovalPolicy,
}

impl ExecutionPolicy {
  pub fn new(
    sandbox_mode: SandboxMode,
    network_access: NetworkAccess,
    approval_policy: ApprovalPolicy,
  ) -> Result<Self, ExecutionPolicyError> {
    if sandbox_mode == SandboxMode::DangerFullAccess && network_access != NetworkAccess::Allowed {
      return Err(ExecutionPolicyError::DangerFullAccessRequiresNetwork);
    }

    return Ok(Self {
      sandbox_mode,
      network_access,
      approval_policy,
    });
  }

  pub fn sandbox_mode(&self) -> SandboxMode {
    return self.sandbox_mode;
  }

  pub fn network_access(&self) -> NetworkAccess {
    return self.network_access;
  }

  pub fn approval_policy(&self) -> ApprovalPolicy {
    return self.approval_policy;
  }
}
