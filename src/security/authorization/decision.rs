use super::{ApprovalRequest, AuthorizationDenied};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationDecision {
  Allowed,
  RequiresApproval(ApprovalRequest),
  Denied(AuthorizationDenied),
}
