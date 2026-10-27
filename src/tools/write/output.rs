use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct WriteOutput {
  path: PathBuf,
  bytes_written: usize,
}

impl WriteOutput {
  pub(super) fn new(path: PathBuf, bytes_written: usize) -> Self {
    return Self {
      path,
      bytes_written,
    };
  }

  pub fn bytes_written(&self) -> usize {
    return self.bytes_written;
  }

  pub fn path(&self) -> &Path {
    return &self.path;
  }
}
