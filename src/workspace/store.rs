use crate::{
  ids::IdGenerator,
  workspace::state::{SCHEMA_VERSION, WorkspaceRecord},
};
use std::{
  fs::{self, File, OpenOptions},
  io::Write,
  path::{Path, PathBuf},
};

const TEMPORARY_FILE_ATTEMPTS: usize = 8;

#[derive(Debug, Eq, PartialEq)]
pub enum WorkspaceStoreError {
  InvalidId,
  NotFound,
  Unreadable,
  Unwritable,
  InvalidWorkspace,
}

pub fn user_directory(home: &Path) -> PathBuf {
  return home.join(".kermet").join("workspaces");
}

pub struct WorkspaceStore {
  directory: PathBuf,
}

impl WorkspaceStore {
  pub fn new(directory: PathBuf) -> Self {
    return Self { directory };
  }

  pub fn save(&self, workspace: &WorkspaceRecord) -> Result<(), WorkspaceStoreError> {
    if fs::create_dir_all(&self.directory).is_err() {
      return Err(WorkspaceStoreError::Unwritable);
    }

    let contents = match serde_json::to_vec_pretty(workspace) {
      Ok(contents) => contents,
      Err(_) => return Err(WorkspaceStoreError::InvalidWorkspace),
    };

    let (temporary_path, mut file) = match self.open_temporary_file() {
      Ok(value) => value,
      Err(error) => return Err(error),
    };

    if file.write_all(&contents).is_err() || file.sync_all().is_err() {
      let _: std::io::Result<()> = fs::remove_file(&temporary_path);

      return Err(WorkspaceStoreError::Unwritable);
    }

    if fs::rename(&temporary_path, self.path(workspace.id())).is_err() {
      let _: std::io::Result<()> = fs::remove_file(&temporary_path);

      return Err(WorkspaceStoreError::Unwritable);
    }

    return Ok(());
  }

  pub fn load(&self, id: &str) -> Result<WorkspaceRecord, WorkspaceStoreError> {
    if IdGenerator::validate("workspace", id).is_err() {
      return Err(WorkspaceStoreError::InvalidId);
    }

    let contents = match fs::read(self.path(id)) {
      Ok(contents) => contents,
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
        return Err(WorkspaceStoreError::NotFound);
      }
      Err(_) => return Err(WorkspaceStoreError::Unreadable),
    };

    let workspace = match serde_json::from_slice::<WorkspaceRecord>(&contents) {
      Ok(workspace) => workspace,
      Err(_) => return Err(WorkspaceStoreError::InvalidWorkspace),
    };

    if workspace.version != SCHEMA_VERSION || workspace.id != id {
      return Err(WorkspaceStoreError::InvalidWorkspace);
    }

    return Ok(workspace);
  }

  pub fn list(&self) -> Result<Vec<WorkspaceRecord>, WorkspaceStoreError> {
    let entries = match fs::read_dir(&self.directory) {
      Ok(entries) => entries,
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
      Err(_) => return Err(WorkspaceStoreError::Unreadable),
    };
    let mut workspaces = Vec::new();

    for entry in entries {
      let entry = match entry {
        Ok(entry) => entry,
        Err(_) => return Err(WorkspaceStoreError::Unreadable),
      };
      let file_type = match entry.file_type() {
        Ok(file_type) => file_type,
        Err(_) => return Err(WorkspaceStoreError::Unreadable),
      };

      if !file_type.is_file() {
        continue;
      }

      let file_name = entry.file_name();
      let Some(file_name) = file_name.to_str() else {
        return Err(WorkspaceStoreError::InvalidWorkspace);
      };
      let Some(id) = file_name.strip_suffix(".json") else {
        continue;
      };
      let workspace = match self.load(id) {
        Ok(workspace) => workspace,
        Err(error) => return Err(error),
      };

      workspaces.push(workspace);
    }

    workspaces.sort_by(|left, right| return left.id().cmp(right.id()));

    return Ok(workspaces);
  }

  fn path(&self, id: &str) -> PathBuf {
    return self.directory.join(format!("{id}.json"));
  }

  fn open_temporary_file(&self) -> Result<(PathBuf, File), WorkspaceStoreError> {
    for _ in 0..TEMPORARY_FILE_ATTEMPTS {
      let nonce = rand::random::<u64>();
      let path = self
        .directory
        .join(format!(".workspace-{}-{nonce}.tmp", std::process::id()));

      let file = match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
        Err(_) => return Err(WorkspaceStoreError::Unwritable),
      };

      return Ok((path, file));
    }

    return Err(WorkspaceStoreError::Unwritable);
  }
}
