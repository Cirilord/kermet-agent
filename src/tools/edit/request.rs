use std::path::{Path, PathBuf};

pub const MAX_CONTENT_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug)]
pub struct EditRequest {
  path: PathBuf,
  old_text: String,
  new_text: String,
}

impl EditRequest {
  pub fn new(path: PathBuf, old_text: String, new_text: String) -> Result<Self, EditRequestError> {
    if path.as_os_str().is_empty() {
      return Err(EditRequestError::EmptyPath);
    }
    if old_text.is_empty() {
      return Err(EditRequestError::EmptyOldText);
    }
    if old_text.len() > MAX_CONTENT_BYTES || new_text.len() > MAX_CONTENT_BYTES {
      return Err(EditRequestError::InputTooLarge {
        limit: MAX_CONTENT_BYTES,
      });
    }

    return Ok(Self {
      path,
      old_text,
      new_text,
    });
  }

  pub fn new_text(&self) -> &str {
    return &self.new_text;
  }

  pub fn old_text(&self) -> &str {
    return &self.old_text;
  }

  pub fn path(&self) -> &Path {
    return &self.path;
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EditRequestError {
  EmptyPath,
  EmptyOldText,
  InputTooLarge { limit: usize },
}
