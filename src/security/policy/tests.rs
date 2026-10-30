use super::super::sandbox::{NetworkAccess, SandboxMode};
use super::{ApprovalPolicy, ExecutionPolicy, ExecutionPolicyError};

#[test]
fn exposes_stable_approval_policy_identifiers() -> () {
  assert_eq!(ApprovalPolicy::OnRequest.identifier(), "on-request");
  assert_eq!(ApprovalPolicy::Never.identifier(), "never");

  return;
}

#[test]
fn creates_an_execution_policy_from_independent_controls() -> () {
  let policy = ExecutionPolicy::new(
    SandboxMode::WorkspaceWrite,
    NetworkAccess::Denied,
    ApprovalPolicy::OnRequest,
  )
  .expect("the execution policy should be valid");

  assert_eq!(policy.sandbox_mode(), SandboxMode::WorkspaceWrite);
  assert_eq!(policy.network_access(), NetworkAccess::Denied);
  assert_eq!(policy.approval_policy(), ApprovalPolicy::OnRequest);

  return;
}

#[test]
fn rejects_danger_full_access_without_network_access() -> () {
  let result = ExecutionPolicy::new(
    SandboxMode::DangerFullAccess,
    NetworkAccess::Denied,
    ApprovalPolicy::Never,
  );

  assert_eq!(
    result,
    Err(ExecutionPolicyError::DangerFullAccessRequiresNetwork)
  );

  return;
}
