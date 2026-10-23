use super::{WorkspaceRecord, WorkspaceStore, WorkspaceStoreError, user_directory};
use std::{
  fs,
  path::{Path, PathBuf},
};

fn temporary_directory(name: &str) -> PathBuf {
  let directory = std::env::temp_dir().join(format!("kermet-{name}-{}", rand::random::<u64>()));
  fs::create_dir(&directory).expect("the temporary directory should be created");

  return directory;
}

#[test]
fn saves_and_loads_a_workspace_record() -> () {
  let directory = temporary_directory("workspace-store");
  let store = WorkspaceStore::new(directory.clone());
  let workspace = WorkspaceRecord::new(
    "workspace-0123456789abcdef".to_owned(),
    PathBuf::from("/workspace"),
  );

  store
    .save(&workspace)
    .expect("the workspace should be saved");
  let loaded = store
    .load(workspace.id())
    .expect("the workspace should be loaded");

  fs::remove_dir_all(directory).expect("the temporary directory should be removed");

  assert_eq!(loaded.id(), "workspace-0123456789abcdef");
  assert_eq!(loaded.path(), Path::new("/workspace"));

  return;
}

#[test]
fn lists_saved_workspace_records_by_identifier() -> () {
  let directory = temporary_directory("workspace-list");
  let store = WorkspaceStore::new(directory.clone());
  let first = WorkspaceRecord::new(
    "workspace-aaaaaaaaaaaaaaaa".to_owned(),
    PathBuf::from("/first"),
  );
  let second = WorkspaceRecord::new(
    "workspace-bbbbbbbbbbbbbbbb".to_owned(),
    PathBuf::from("/second"),
  );

  store
    .save(&second)
    .expect("the second workspace should be saved");
  store
    .save(&first)
    .expect("the first workspace should be saved");
  let workspaces = store.list().expect("the workspaces should be listed");

  fs::remove_dir_all(directory).expect("the temporary directory should be removed");

  assert_eq!(workspaces.len(), 2);
  assert_eq!(workspaces[0].id(), "workspace-aaaaaaaaaaaaaaaa");
  assert_eq!(workspaces[1].id(), "workspace-bbbbbbbbbbbbbbbb");

  return;
}

#[test]
fn rejects_an_invalid_workspace_identifier() -> () {
  let directory = temporary_directory("workspace-id");
  let store = WorkspaceStore::new(directory.clone());

  let error = store
    .load("../workspace")
    .expect_err("the identifier should be rejected");

  fs::remove_dir_all(directory).expect("the temporary directory should be removed");

  assert_eq!(error, WorkspaceStoreError::InvalidId);

  return;
}

#[test]
fn derives_a_workspace_directory_from_the_user_home() -> () {
  assert_eq!(
    user_directory(Path::new("/home/user")),
    PathBuf::from("/home/user/.kermet/workspaces")
  );

  return;
}
