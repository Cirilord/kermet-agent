use super::{
  NetworkAccess, ResourceLimits, Sandbox, SandboxError, SandboxMode, SandboxPlatform,
  SandboxPolicy, SandboxPolicyError, SandboxRequest, SandboxRequestError, SandboxRunner,
  UnavailableSandbox,
};
use std::{path::PathBuf, time::Duration};

#[cfg(target_os = "macos")]
use super::environments::MacOsSandbox;

#[test]
fn creates_the_sandbox_for_the_current_environment() -> () {
  let sandbox = Sandbox::current();

  assert_eq!(sandbox.platform(), SandboxPlatform::current());

  return;
}

#[test]
fn creates_a_workspace_write_policy_with_network_denied() -> () {
  let policy = SandboxPolicy::workspace_write(PathBuf::from("/workspace"))
    .expect("the policy should be valid");

  assert_eq!(policy.mode(), SandboxMode::WorkspaceWrite);
  assert_eq!(policy.workspace(), std::path::Path::new("/workspace"));
  assert_eq!(policy.network_access(), NetworkAccess::Denied);
  assert_eq!(policy.resource_limits(), ResourceLimits::default());

  return;
}

#[test]
fn rejects_an_empty_workspace() -> () {
  let result = SandboxPolicy::workspace_write(PathBuf::new());

  assert_eq!(result, Err(SandboxPolicyError::InvalidWorkspace));

  return;
}

#[test]
fn rejects_empty_resource_limits() -> () {
  let result = ResourceLimits::new(Duration::ZERO, 0);

  assert_eq!(result, Err(SandboxPolicyError::InvalidResourceLimits));

  return;
}

#[test]
fn creates_a_request_with_explicit_environment() -> () {
  let request = SandboxRequest::new("git".to_owned(), vec!["status".to_owned()])
    .expect("the request should be valid")
    .with_environment("PATH".to_owned(), "/usr/bin".to_owned())
    .expect("the environment name should be valid");

  assert_eq!(request.program(), "git");
  assert_eq!(request.arguments(), ["status"]);
  assert_eq!(
    request.environment().get("PATH"),
    Some(&"/usr/bin".to_owned())
  );

  return;
}

#[test]
fn rejects_invalid_request_values() -> () {
  let empty_program = SandboxRequest::new("  ".to_owned(), Vec::new());
  let invalid_environment = SandboxRequest::new("git".to_owned(), Vec::new())
    .expect("the request should be valid")
    .with_environment("PATH-OVERRIDE".to_owned(), "/usr/bin".to_owned());

  assert_eq!(empty_program, Err(SandboxRequestError::EmptyProgram));
  assert_eq!(
    invalid_environment,
    Err(SandboxRequestError::InvalidEnvironmentName)
  );

  return;
}

#[test]
fn unavailable_sandbox_fails_closed() -> () {
  let sandbox = UnavailableSandbox::new();
  let policy = SandboxPolicy::workspace_write(PathBuf::from("/workspace"))
    .expect("the policy should be valid");
  let request = SandboxRequest::new("git".to_owned(), vec!["status".to_owned()])
    .expect("the request should be valid");
  let result = sandbox.execute(&policy, request);

  assert!(matches!(
    result,
    Err(SandboxError::Unavailable(platform)) if platform == SandboxPlatform::current()
  ));

  return;
}

#[cfg(target_os = "macos")]
#[test]
fn macos_workspace_write_profile_limits_access_to_the_workspace() -> () {
  let policy = SandboxPolicy::workspace_write(PathBuf::from("/workspace"))
    .expect("the policy should be valid");
  let profile = MacOsSandbox::profile(&policy).expect("the profile should be valid");

  assert!(profile.contains("(deny default)"));
  assert!(profile.contains("(allow file-read* (subpath \"/workspace\"))"));
  assert!(profile.contains("(allow file-write* (subpath \"/workspace\"))"));
  assert!(!profile.contains("(allow network*)"));

  return;
}

#[cfg(target_os = "macos")]
#[test]
fn macos_danger_full_access_profile_allows_default_operations() -> () {
  let policy = SandboxPolicy::danger_full_access(PathBuf::from("/workspace"))
    .expect("the policy should be valid");
  let profile = MacOsSandbox::profile(&policy).expect("the profile should be valid");

  assert_eq!(profile, "(version 1) (allow default)");

  return;
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires a macOS host that permits Seatbelt execution"]
fn macos_sandbox_executes_a_workspace_process() -> () {
  let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let policy =
    SandboxPolicy::workspace_write(workspace.clone()).expect("the policy should be valid");
  let request =
    SandboxRequest::new("/bin/pwd".to_owned(), Vec::new()).expect("the request should be valid");
  let output = MacOsSandbox::new()
    .execute(&policy, request)
    .expect("the sandbox should execute the process");
  let expected_stdout = format!("{}\n", workspace.display());

  assert_eq!(output.exit_code(), Some(0));
  assert_eq!(output.stdout(), expected_stdout.as_bytes());
  assert!(output.stderr().is_empty());

  return;
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires a macOS host that permits Seatbelt execution"]
fn macos_sandbox_denies_reading_an_external_file() -> () {
  let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let policy = SandboxPolicy::workspace_write(workspace).expect("the policy should be valid");
  let request = SandboxRequest::new("/bin/cat".to_owned(), vec!["/etc/hosts".to_owned()])
    .expect("the request should be valid");
  let output = MacOsSandbox::new()
    .execute(&policy, request)
    .expect("the sandbox should execute the process");

  assert_ne!(output.exit_code(), Some(0));
  assert!(output.stdout().is_empty());

  return;
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires a macOS host that permits Seatbelt execution"]
fn macos_sandbox_allows_writing_inside_the_workspace() -> () {
  let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let target = workspace.join("target/kermet-sandbox-test-file");
  let policy = SandboxPolicy::workspace_write(workspace).expect("the policy should be valid");
  let request = SandboxRequest::new(
    "/usr/bin/touch".to_owned(),
    vec![target.display().to_string()],
  )
  .expect("the request should be valid");
  let output = MacOsSandbox::new()
    .execute(&policy, request)
    .expect("the sandbox should execute the process");

  assert_eq!(output.exit_code(), Some(0));
  assert!(target.exists());
  std::fs::remove_file(target).expect("the test file should be removed");

  return;
}
