use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ReadOutput {
  path: PathBuf,
  contents: Vec<u8>,
}

impl ReadOutput {
  pub(super) fn new(path: PathBuf, contents: Vec<u8>) -> Self {
    return Self { path, contents };
  }

  pub fn contents(&self) -> &[u8] {
    return &self.contents;
  }

  pub fn path(&self) -> &Path {
    return &self.path;
  }
}
