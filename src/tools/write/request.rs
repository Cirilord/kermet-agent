use std::path::{Path, PathBuf};

pub const MAX_CONTENT_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug)]
pub struct WriteRequest {
  path: PathBuf,
  contents: String,
}

impl WriteRequest {
  pub fn new(path: PathBuf, contents: String) -> Result<Self, WriteRequestError> {
    if path.as_os_str().is_empty() {
      return Err(WriteRequestError::EmptyPath);
    }
    if contents.len() > MAX_CONTENT_BYTES {
      return Err(WriteRequestError::TooLarge {
        limit: MAX_CONTENT_BYTES,
      });
    }

    return Ok(Self { path, contents });
  }

  pub fn contents(&self) -> &str {
    return &self.contents;
  }

  pub fn path(&self) -> &Path {
    return &self.path;
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum WriteRequestError {
  EmptyPath,
  TooLarge { limit: usize },
}
