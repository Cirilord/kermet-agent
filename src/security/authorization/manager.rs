use super::super::{
  ApprovalPolicy, ExecutionPolicy,
  sandbox::{NetworkAccess, SandboxMode},
};
use super::{
  ApprovalRequest, AuthorizationDecision, AuthorizationDenied, AuthorizationError,
  ExecutionRequirement,
};

#[derive(Clone, Debug, Default)]
pub struct AuthorizationManager;

impl AuthorizationManager {
  pub fn new() -> Self {
    return Self;
  }

  pub fn evaluate(
    &self,
    policy: &ExecutionPolicy,
    requirement: ExecutionRequirement,
  ) -> AuthorizationDecision {
    if allows(policy, requirement) {
      return AuthorizationDecision::Allowed;
    }

    let decision = match policy.approval_policy() {
      ApprovalPolicy::OnRequest => {
        AuthorizationDecision::RequiresApproval(ApprovalRequest::new(requirement))
      }
      ApprovalPolicy::Never => {
        AuthorizationDecision::Denied(AuthorizationDenied::new(*policy, requirement))
      }
    };

    return decision;
  }

  pub fn authorize(
    &self,
    policy: &ExecutionPolicy,
    requirement: ExecutionRequirement,
  ) -> Result<(), AuthorizationError> {
    let result = match self.evaluate(policy, requirement) {
      AuthorizationDecision::Allowed => Ok(()),
      AuthorizationDecision::RequiresApproval(request) => {
        Err(AuthorizationError::ApprovalRequired(request))
      }
      AuthorizationDecision::Denied(denied) => Err(AuthorizationError::Denied(denied)),
    };

    return result;
  }
}

fn allows(policy: &ExecutionPolicy, requirement: ExecutionRequirement) -> bool {
  let is_allowed = sandbox_mode_allows(policy.sandbox_mode(), requirement.sandbox_mode())
    && network_access_allows(policy.network_access(), requirement.network_access());

  return is_allowed;
}

fn sandbox_mode_allows(active: SandboxMode, required: SandboxMode) -> bool {
  let is_allowed = match active {
    SandboxMode::ReadOnly => required == SandboxMode::ReadOnly,
    SandboxMode::WorkspaceWrite => {
      matches!(
        required,
        SandboxMode::ReadOnly | SandboxMode::WorkspaceWrite
      )
    }
    SandboxMode::DangerFullAccess => true,
  };

  return is_allowed;
}

fn network_access_allows(active: NetworkAccess, required: NetworkAccess) -> bool {
  let is_allowed = active == NetworkAccess::Allowed || required == NetworkAccess::Denied;

  return is_allowed;
}
