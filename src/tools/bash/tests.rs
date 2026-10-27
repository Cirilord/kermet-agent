use super::{Bash, BashError, BashRequest, BashRequestError};
use crate::{
  agent::{PermissionMode, TaskRequest},
  security::PermissionManager,
  workspace::Workspace,
};
use std::{
  fs,
  sync::atomic::{AtomicU64, Ordering},
  time::{Duration, SystemTime, UNIX_EPOCH},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

#[test]
fn rejects_an_empty_command() -> () {
  let error = BashRequest::new("  ".to_owned()).expect_err("the command should be rejected");

  assert_eq!(error, BashRequestError::EmptyCommand);

  return;
}

#[test]
fn denies_execution_without_expanded_access() -> () {
  let task = task(PermissionMode::WorkspaceWrite);
  let request = BashRequest::new("printf denied".to_owned()).expect("the request should be valid");
  let error = Bash::new(PermissionManager::new())
    .execute(&task, request)
    .expect_err("the permission should be denied");

  assert!(matches!(error, BashError::PermissionDenied(_)));

  return;
}

#[test]
fn runs_a_command_in_the_workspace() -> () {
  let task = task(PermissionMode::ExpandedAccess);
  let request =
    BashRequest::new("printf '%s' \"$PWD\"".to_owned()).expect("the request should be valid");
  let output = Bash::new(PermissionManager::new())
    .execute(&task, request)
    .expect("the command should run");

  assert_eq!(output.exit_code(), Some(0));
  assert!(!output.timed_out());
  assert_eq!(
    output.stdout(),
    task.workspace().path().as_os_str().as_encoded_bytes()
  );
  assert!(output.stderr().is_empty());

  return;
}

#[test]
fn preserves_a_nonzero_exit_code_and_standard_error() -> () {
  let task = task(PermissionMode::ExpandedAccess);
  let request =
    BashRequest::new("printf failure >&2; exit 7".to_owned()).expect("the request should be valid");
  let output = Bash::new(PermissionManager::new())
    .execute(&task, request)
    .expect("the process result should be captured");

  assert_eq!(output.exit_code(), Some(7));
  assert_eq!(output.stderr(), b"failure");

  return;
}

#[test]
fn truncates_large_standard_output() -> () {
  let task = task(PermissionMode::ExpandedAccess);
  let request =
    BashRequest::new("printf 'x%.0s' {1..40000}".to_owned()).expect("the request should be valid");
  let output = Bash::new(PermissionManager::new())
    .execute(&task, request)
    .expect("the process result should be captured");

  assert_eq!(output.stdout().len(), 32 * 1024);
  assert!(output.stdout_truncated());

  return;
}

#[test]
fn kills_a_command_that_exceeds_its_timeout() -> () {
  let task = task(PermissionMode::ExpandedAccess);
  let request = BashRequest::new("sleep 1".to_owned()).expect("the request should be valid");
  let output = Bash::with_timeout(PermissionManager::new(), Duration::from_millis(20))
    .execute(&task, request)
    .expect("the timed out process should be captured");

  assert!(output.timed_out());

  return;
}

fn task(permission_mode: PermissionMode) -> TaskRequest {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("the clock should be valid")
    .as_nanos();
  let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
  let workspace_path = std::env::temp_dir().join(format!(
    "kermet-bash-{}-{timestamp}-{sequence}",
    std::process::id()
  ));
  fs::create_dir(&workspace_path).expect("the fixture should be created");
  let workspace = Workspace::new(
    fs::canonicalize(&workspace_path).expect("the fixture should be canonical"),
    false,
  );
  let request = TaskRequest::new("run a command".to_owned(), workspace, permission_mode)
    .expect("the task should be valid");

  return request;
}
