use std::path::{Path, PathBuf};

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
