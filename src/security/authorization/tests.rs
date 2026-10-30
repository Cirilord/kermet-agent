use super::super::{
  ApprovalPolicy, ExecutionPolicy,
  sandbox::{NetworkAccess, SandboxMode},
};
use super::{AuthorizationDecision, AuthorizationManager, ExecutionRequirement};

#[test]
fn allows_a_read_only_requirement_in_read_only_mode() -> () {
  let manager = AuthorizationManager::new();
  let policy = policy(SandboxMode::ReadOnly, ApprovalPolicy::OnRequest);
  let requirement = requirement(SandboxMode::ReadOnly, NetworkAccess::Denied);

  assert_eq!(
    manager.evaluate(&policy, requirement),
    AuthorizationDecision::Allowed
  );

  return;
}

#[test]
fn allows_a_workspace_write_requirement_in_workspace_write_mode() -> () {
  let manager = AuthorizationManager::new();
  let policy = policy(SandboxMode::WorkspaceWrite, ApprovalPolicy::OnRequest);
  let requirement = requirement(SandboxMode::WorkspaceWrite, NetworkAccess::Denied);

  assert_eq!(
    manager.evaluate(&policy, requirement),
    AuthorizationDecision::Allowed
  );

  return;
}

#[test]
fn requests_approval_for_a_requirement_outside_the_active_sandbox() -> () {
  let manager = AuthorizationManager::new();
  let policy = policy(SandboxMode::ReadOnly, ApprovalPolicy::OnRequest);
  let requirement = requirement(SandboxMode::WorkspaceWrite, NetworkAccess::Denied);
  let decision = manager.evaluate(&policy, requirement);

  let AuthorizationDecision::RequiresApproval(request) = decision else {
    panic!("a workspace write requirement should require approval");
  };
  assert_eq!(request.requirement(), requirement);

  return;
}

#[test]
fn denies_a_requirement_when_approval_is_disabled() -> () {
  let manager = AuthorizationManager::new();
  let policy = policy(SandboxMode::ReadOnly, ApprovalPolicy::Never);
  let requirement = requirement(SandboxMode::WorkspaceWrite, NetworkAccess::Denied);
  let decision = manager.evaluate(&policy, requirement);

  let AuthorizationDecision::Denied(denied) = decision else {
    panic!("the requirement should be denied");
  };
  assert_eq!(denied.policy(), policy);
  assert_eq!(denied.requirement(), requirement);

  return;
}

#[test]
fn requests_approval_for_network_access_outside_the_active_policy() -> () {
  let manager = AuthorizationManager::new();
  let policy = policy(SandboxMode::WorkspaceWrite, ApprovalPolicy::OnRequest);
  let requirement = requirement(SandboxMode::ReadOnly, NetworkAccess::Allowed);

  assert!(matches!(
    manager.evaluate(&policy, requirement),
    AuthorizationDecision::RequiresApproval(request) if request.requirement() == requirement
  ));

  return;
}

fn policy(sandbox_mode: SandboxMode, approval_policy: ApprovalPolicy) -> ExecutionPolicy {
  let policy = ExecutionPolicy::new(sandbox_mode, NetworkAccess::Denied, approval_policy)
    .expect("the execution policy should be valid");

  return policy;
}

fn requirement(sandbox_mode: SandboxMode, network_access: NetworkAccess) -> ExecutionRequirement {
  let requirement = ExecutionRequirement::new(sandbox_mode, network_access);

  return requirement;
}
