use super::{Write, WriteError, WriteRequest, WriteRequestError};
use crate::{
  agent::{PermissionMode, TaskRequest},
  security::PermissionManager,
  workspace::Workspace,
};
use std::{
  fs,
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn writes_a_new_file_inside_the_workspace() -> () {
  let workspace_path = unique_path("new-file");
  fs::create_dir(&workspace_path).expect("the workspace should be created");
  let task = task(&workspace_path, PermissionMode::WorkspaceWrite);
  let request = WriteRequest::new(PathBuf::from("notes.txt"), "Kermet".to_owned())
    .expect("the request should be valid");
  let output = Write::new(PermissionManager::new())
    .execute(&task, request)
    .expect("the file should be written");

  assert_eq!(output.path(), task.workspace().path().join("notes.txt"));
  assert_eq!(output.bytes_written(), 6);
  assert_eq!(
    fs::read_to_string(workspace_path.join("notes.txt")).expect("the file should be readable"),
    "Kermet"
  );
  fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");

  return;
}

#[test]
fn replaces_an_existing_file() -> () {
  let workspace_path = unique_path("replace-file");
  fs::create_dir(&workspace_path).expect("the workspace should be created");
  fs::write(workspace_path.join("notes.txt"), "old").expect("the fixture should be written");
  let task = task(&workspace_path, PermissionMode::WorkspaceWrite);
  let request = WriteRequest::new(PathBuf::from("notes.txt"), "new".to_owned())
    .expect("the request should be valid");

  Write::new(PermissionManager::new())
    .execute(&task, request)
    .expect("the file should be replaced");

  assert_eq!(
    fs::read_to_string(workspace_path.join("notes.txt")).expect("the file should be readable"),
    "new"
  );
  fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");

  return;
}

#[test]
fn rejects_a_path_outside_the_workspace() -> () {
  let root = unique_path("outside-workspace");
  let workspace_path = root.join("workspace");
  fs::create_dir_all(&workspace_path).expect("the workspace should be created");
  let task = task(&workspace_path, PermissionMode::WorkspaceWrite);
  let request = WriteRequest::new(PathBuf::from("../outside.txt"), "outside".to_owned())
    .expect("the request should be valid");
  let error = Write::new(PermissionManager::new())
    .execute(&task, request)
    .expect_err("the path should be rejected");

  assert!(matches!(error, WriteError::OutsideWorkspace { .. }));
  fs::remove_dir_all(&root).expect("the fixture should be removed");

  return;
}

#[cfg(unix)]
#[test]
fn rejects_a_parent_symlink_that_escapes_the_workspace() -> () {
  use std::os::unix::fs::symlink;

  let root = unique_path("symlink");
  let workspace_path = root.join("workspace");
  let outside_path = root.join("outside");
  fs::create_dir_all(&workspace_path).expect("the workspace should be created");
  fs::create_dir(&outside_path).expect("the outside directory should be created");
  symlink(&outside_path, workspace_path.join("escape")).expect("the symlink should be created");
  let task = task(&workspace_path, PermissionMode::WorkspaceWrite);
  let request = WriteRequest::new(PathBuf::from("escape/notes.txt"), "outside".to_owned())
    .expect("the request should be valid");
  let error = Write::new(PermissionManager::new())
    .execute(&task, request)
    .expect_err("the path should be rejected");

  assert!(matches!(error, WriteError::OutsideWorkspace { .. }));
  fs::remove_dir_all(&root).expect("the fixture should be removed");

  return;
}

#[cfg(unix)]
#[test]
fn rejects_a_target_symlink_that_escapes_the_workspace() -> () {
  use std::os::unix::fs::symlink;

  let root = unique_path("target-symlink");
  let workspace_path = root.join("workspace");
  let outside_path = root.join("outside.txt");
  fs::create_dir_all(&workspace_path).expect("the workspace should be created");
  fs::write(&outside_path, "outside").expect("the outside file should be written");
  symlink(&outside_path, workspace_path.join("escape.txt")).expect("the symlink should be created");
  let task = task(&workspace_path, PermissionMode::WorkspaceWrite);
  let request = WriteRequest::new(PathBuf::from("escape.txt"), "unsafe".to_owned())
    .expect("the request should be valid");
  let error = Write::new(PermissionManager::new())
    .execute(&task, request)
    .expect_err("the path should be rejected");

  assert!(matches!(error, WriteError::OutsideWorkspace { .. }));
  assert_eq!(
    fs::read_to_string(&outside_path).expect("the outside file should be readable"),
    "outside"
  );
  fs::remove_dir_all(&root).expect("the fixture should be removed");

  return;
}

#[test]
fn denies_writes_in_read_only_mode() -> () {
  let workspace_path = unique_path("permission");
  fs::create_dir(&workspace_path).expect("the workspace should be created");
  let task = task(&workspace_path, PermissionMode::ReadOnly);
  let request = WriteRequest::new(PathBuf::from("notes.txt"), "denied".to_owned())
    .expect("the request should be valid");
  let error = Write::new(PermissionManager::new())
    .execute(&task, request)
    .expect_err("the write should be denied");

  assert!(matches!(error, WriteError::PermissionDenied(_)));
  fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");

  return;
}

#[test]
fn rejects_an_empty_path() -> () {
  let error = WriteRequest::new(PathBuf::new(), "contents".to_owned())
    .expect_err("the path should be rejected");

  assert_eq!(error, WriteRequestError::EmptyPath);

  return;
}

#[test]
fn rejects_content_larger_than_the_limit() -> () {
  let contents = "a".repeat(256 * 1024 + 1);
  let error = WriteRequest::new(PathBuf::from("notes.txt"), contents)
    .expect_err("the contents should be rejected");

  assert_eq!(error, WriteRequestError::TooLarge { limit: 256 * 1024 });

  return;
}

fn task(workspace_path: &std::path::Path, permission_mode: PermissionMode) -> TaskRequest {
  let workspace = Workspace::new(
    fs::canonicalize(workspace_path).expect("the workspace should be canonical"),
    false,
  );
  let task = TaskRequest::new("write file".to_owned(), workspace, permission_mode)
    .expect("the task should be valid");

  return task;
}

fn unique_path(kind: &str) -> PathBuf {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("the clock should be valid")
    .as_nanos();

  return std::env::temp_dir().join(format!(
    "kermet-write-{kind}-{}-{timestamp}",
    std::process::id()
  ));
}
