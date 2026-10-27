use super::{Edit, EditError, EditRequest, EditRequestError};
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
fn replaces_one_exact_text_occurrence() -> () {
  let workspace_path = unique_path("replace");
  fs::create_dir(&workspace_path).expect("the workspace should be created");
  fs::write(workspace_path.join("notes.txt"), "before target after")
    .expect("the fixture should be written");
  let task = task(&workspace_path, PermissionMode::WorkspaceWrite);
  let request = EditRequest::new(
    PathBuf::from("notes.txt"),
    "target".to_owned(),
    "replacement".to_owned(),
  )
  .expect("the request should be valid");
  let output = Edit::new(PermissionManager::new())
    .execute(&task, request)
    .expect("the file should be edited");

  assert_eq!(output.path(), task.workspace().path().join("notes.txt"));
  assert_eq!(output.bytes_written(), 24);
  assert_eq!(
    fs::read_to_string(workspace_path.join("notes.txt")).expect("the file should be readable"),
    "before replacement after"
  );
  fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");

  return;
}

#[test]
fn rejects_missing_old_text() -> () {
  let workspace_path = unique_path("missing-text");
  fs::create_dir(&workspace_path).expect("the workspace should be created");
  fs::write(workspace_path.join("notes.txt"), "contents").expect("the fixture should be written");
  let task = task(&workspace_path, PermissionMode::WorkspaceWrite);
  let request = EditRequest::new(
    PathBuf::from("notes.txt"),
    "missing".to_owned(),
    "replacement".to_owned(),
  )
  .expect("the request should be valid");
  let error = Edit::new(PermissionManager::new())
    .execute(&task, request)
    .expect_err("the edit should be rejected");

  assert!(matches!(error, EditError::OldTextNotFound));
  assert_eq!(
    fs::read_to_string(workspace_path.join("notes.txt")).expect("the file should be readable"),
    "contents"
  );
  fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");

  return;
}

#[test]
fn rejects_ambiguous_old_text() -> () {
  let workspace_path = unique_path("ambiguous-text");
  fs::create_dir(&workspace_path).expect("the workspace should be created");
  fs::write(workspace_path.join("notes.txt"), "target target")
    .expect("the fixture should be written");
  let task = task(&workspace_path, PermissionMode::WorkspaceWrite);
  let request = EditRequest::new(
    PathBuf::from("notes.txt"),
    "target".to_owned(),
    "replacement".to_owned(),
  )
  .expect("the request should be valid");
  let error = Edit::new(PermissionManager::new())
    .execute(&task, request)
    .expect_err("the edit should be rejected");

  assert!(matches!(error, EditError::OldTextAmbiguous));
  fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");

  return;
}

#[test]
fn rejects_an_empty_old_text() -> () {
  let error = EditRequest::new(
    PathBuf::from("notes.txt"),
    String::new(),
    "replacement".to_owned(),
  )
  .expect_err("the old text should be rejected");

  assert_eq!(error, EditRequestError::EmptyOldText);

  return;
}

#[test]
fn denies_edits_in_read_only_mode() -> () {
  let workspace_path = unique_path("permission");
  fs::create_dir(&workspace_path).expect("the workspace should be created");
  fs::write(workspace_path.join("notes.txt"), "target").expect("the fixture should be written");
  let task = task(&workspace_path, PermissionMode::ReadOnly);
  let request = EditRequest::new(
    PathBuf::from("notes.txt"),
    "target".to_owned(),
    "replacement".to_owned(),
  )
  .expect("the request should be valid");
  let error = Edit::new(PermissionManager::new())
    .execute(&task, request)
    .expect_err("the edit should be denied");

  assert!(matches!(error, EditError::PermissionDenied(_)));
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
  fs::write(&outside_path, "target").expect("the outside file should be written");
  symlink(&outside_path, workspace_path.join("escape.txt")).expect("the symlink should be created");
  let task = task(&workspace_path, PermissionMode::WorkspaceWrite);
  let request = EditRequest::new(
    PathBuf::from("escape.txt"),
    "target".to_owned(),
    "replacement".to_owned(),
  )
  .expect("the request should be valid");
  let error = Edit::new(PermissionManager::new())
    .execute(&task, request)
    .expect_err("the edit should be rejected");

  assert!(matches!(error, EditError::OutsideWorkspace { .. }));
  assert_eq!(
    fs::read_to_string(&outside_path).expect("the outside file should be readable"),
    "target"
  );
  fs::remove_dir_all(&root).expect("the fixture should be removed");

  return;
}

fn task(workspace_path: &std::path::Path, permission_mode: PermissionMode) -> TaskRequest {
  let workspace = Workspace::new(
    fs::canonicalize(workspace_path).expect("the workspace should be canonical"),
    false,
  );
  let task = TaskRequest::new("edit file".to_owned(), workspace, permission_mode)
    .expect("the task should be valid");

  return task;
}

fn unique_path(kind: &str) -> PathBuf {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("the clock should be valid")
    .as_nanos();

  return std::env::temp_dir().join(format!(
    "kermet-edit-{kind}-{}-{timestamp}",
    std::process::id()
  ));
}
