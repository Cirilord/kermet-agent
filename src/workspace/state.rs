use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub(super) const SCHEMA_VERSION: u8 = 1;

#[derive(Debug)]
pub struct Workspace {
  path: PathBuf,
  has_git_metadata: bool,
}

impl Workspace {
  pub(crate) fn new(path: PathBuf, has_git_metadata: bool) -> Self {
    return Self {
      path,
      has_git_metadata,
    };
  }

  pub fn path(&self) -> &Path {
    return &self.path;
  }

  pub fn has_git_metadata(&self) -> bool {
    return self.has_git_metadata;
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkspaceRecord {
  pub(super) version: u8,
  pub(super) id: String,
  pub(super) path: PathBuf,
}

impl WorkspaceRecord {
  pub fn new(id: String, path: PathBuf) -> Self {
    return Self {
      version: SCHEMA_VERSION,
      id,
      path,
    };
  }

  pub fn id(&self) -> &str {
    return &self.id;
  }

  pub fn path(&self) -> &Path {
    return &self.path;
  }
}
