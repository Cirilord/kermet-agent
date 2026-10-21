use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Workspace {
  path: PathBuf,
  has_git_metadata: bool,
}

impl Workspace {
  pub fn open(path: PathBuf) -> Result<Self, WorkspaceError> {
    let metadata = match std::fs::metadata(&path) {
      Ok(metadata) => metadata,
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
        return Err(WorkspaceError::NotFound(path));
      }
      Err(source) => {
        return Err(WorkspaceError::Metadata { path, source });
      }
    };

    if !metadata.is_dir() {
      return Err(WorkspaceError::NotDirectory(path));
    }

    let canonical_path = match std::fs::canonicalize(&path) {
      Ok(canonical_path) => canonical_path,
      Err(source) => {
        return Err(WorkspaceError::Canonicalize { path, source });
      }
    };
    let has_git_metadata = canonical_path.join(".git").exists();

    return Ok(Self {
      path: canonical_path,
      has_git_metadata,
    });
  }

  pub fn path(&self) -> &Path {
    return &self.path;
  }

  pub fn has_git_metadata(&self) -> bool {
    return self.has_git_metadata;
  }
}

#[derive(Debug)]
pub enum WorkspaceError {
  NotFound(PathBuf),
  NotDirectory(PathBuf),
  Metadata {
    path: PathBuf,
    source: std::io::Error,
  },
  Canonicalize {
    path: PathBuf,
    source: std::io::Error,
  },
}

impl Display for WorkspaceError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    match *self {
      Self::NotFound(ref path) => {
        return write!(formatter, "workspace does not exist: {}", path.display());
      }
      Self::NotDirectory(ref path) => {
        return write!(
          formatter,
          "workspace is not a directory: {}",
          path.display()
        );
      }
      Self::Metadata {
        ref path,
        ref source,
      } => {
        return write!(
          formatter,
          "failed to read workspace metadata for {}: {source}",
          path.display()
        );
      }
      Self::Canonicalize {
        ref path,
        ref source,
      } => {
        return write!(
          formatter,
          "failed to canonicalize workspace {}: {source}",
          path.display()
        );
      }
    }
  }
}

impl std::error::Error for WorkspaceError {}

#[cfg(test)]
mod tests {
  use super::{Workspace, WorkspaceError};
  use std::time::{SystemTime, UNIX_EPOCH};

  #[test]
  fn rejects_a_missing_workspace() -> () {
    let path = unique_path("missing");

    let result = Workspace::open(path);

    assert!(matches!(result, Err(WorkspaceError::NotFound(_))));

    return;
  }

  #[test]
  fn rejects_a_file_workspace() -> () {
    let path = unique_path("file");
    std::fs::write(&path, "fixture").expect("the fixture should be written");

    let result = Workspace::open(path.clone());

    std::fs::remove_file(path).expect("the fixture should be removed");
    assert!(matches!(result, Err(WorkspaceError::NotDirectory(_))));

    return;
  }

  fn unique_path(kind: &str) -> std::path::PathBuf {
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("the system clock should be after the Unix epoch")
      .as_nanos();
    let name = format!("kermet-{kind}-{}-{timestamp}", std::process::id());

    return std::env::temp_dir().join(name);
  }
}
