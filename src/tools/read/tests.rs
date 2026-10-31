use super::{Read, ReadError, ReadRequest};
use crate::{
  agent::TaskRequest,
  security::{
    ApprovalPolicy, AuthorizationManager, ExecutionPolicy,
    sandbox::{NetworkAccess, SandboxMode},
  },
  workspace::Workspace,
};
use std::{
  fs,
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn reads_a_file_inside_the_workspace() -> () {
  let workspace_path = unique_path("workspace");
  fs::create_dir(&workspace_path).expect("the workspace should be created");
  fs::write(workspace_path.join("notes.txt"), "Kermet").expect("the file should be written");
  let task = task(&workspace_path);
  let output = Read::new(AuthorizationManager::new())
    .execute(&task, ReadRequest::new(PathBuf::from("notes.txt")))
    .expect("the file should be read");

  assert_eq!(output.contents(), b"Kermet");
  fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");

  return;
}

#[test]
fn rejects_a_path_outside_the_workspace() -> () {
  let root = unique_path("outside-workspace");
  let workspace_path = root.join("workspace");
  fs::create_dir_all(&workspace_path).expect("the workspace should be created");
  fs::write(root.join("outside.txt"), "outside").expect("the outside file should be written");
  let task = task(&workspace_path);
  let error = Read::new(AuthorizationManager::new())
    .execute(&task, ReadRequest::new(PathBuf::from("../outside.txt")))
    .expect_err("the path should be rejected");

  assert!(matches!(error, ReadError::OutsideWorkspace { .. }));
  fs::remove_dir_all(&root).expect("the fixture should be removed");

  return;
}

#[test]
fn reads_only_the_requested_lines() -> () {
  let workspace_path = unique_path("lines");
  fs::create_dir(&workspace_path).expect("the workspace should be created");
  fs::write(workspace_path.join("notes.txt"), "one\ntwo\nthree\n")
    .expect("the file should be written");
  let task = task(&workspace_path);
  let request = ReadRequest::with_line_range(PathBuf::from("notes.txt"), 2, 3)
    .expect("the range should be valid");
  let output = Read::new(AuthorizationManager::new())
    .execute(&task, request)
    .expect("the file should be read");

  assert_eq!(output.contents(), b"two\nthree\n");
  fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");

  return;
}

#[test]
fn rejects_a_missing_file() -> () {
  let workspace_path = unique_path("missing");
  fs::create_dir(&workspace_path).expect("the workspace should be created");
  let task = task(&workspace_path);
  let error = Read::new(AuthorizationManager::new())
    .execute(&task, ReadRequest::new(PathBuf::from("missing.txt")))
    .expect_err("the file should be missing");

  assert!(matches!(error, ReadError::Canonicalize { .. }));
  fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");

  return;
}

#[cfg(unix)]
#[test]
fn rejects_a_symlink_that_escapes_the_workspace() -> () {
  use std::os::unix::fs::symlink;

  let root = unique_path("symlink");
  let workspace_path = root.join("workspace");
  let outside_path = root.join("outside.txt");
  fs::create_dir_all(&workspace_path).expect("the workspace should be created");
  fs::write(&outside_path, "outside").expect("the outside file should be written");
  symlink(&outside_path, workspace_path.join("escape.txt")).expect("the symlink should be created");
  let task = task(&workspace_path);
  let error = Read::new(AuthorizationManager::new())
    .execute(&task, ReadRequest::new(PathBuf::from("escape.txt")))
    .expect_err("the symlink should be rejected");

  assert!(matches!(error, ReadError::OutsideWorkspace { .. }));
  fs::remove_dir_all(&root).expect("the fixture should be removed");

  return;
}

fn task(workspace_path: &std::path::Path) -> TaskRequest {
  let workspace = Workspace::new(
    fs::canonicalize(workspace_path).expect("the workspace should be canonical"),
    false,
  );
  let task = TaskRequest::new("read file".to_owned(), workspace, policy())
    .expect("the task should be valid");

  return task;
}

fn policy() -> ExecutionPolicy {
  return ExecutionPolicy::new(
    SandboxMode::ReadOnly,
    NetworkAccess::Denied,
    ApprovalPolicy::Never,
  )
  .expect("the policy should be valid");
}

fn unique_path(kind: &str) -> PathBuf {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("the clock should be valid")
    .as_nanos();

  return std::env::temp_dir().join(format!(
    "kermet-read-{kind}-{}-{timestamp}",
    std::process::id()
  ));
}
