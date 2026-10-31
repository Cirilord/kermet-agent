use super::{WriteOutput, WriteRequest};
use crate::{
  agent::TaskRequest,
  ids::IdGenerator,
  security::{
    AuthorizationError, AuthorizationManager, ExecutionRequirement,
    sandbox::{NetworkAccess, SandboxMode},
  },
};
use std::{
  fs::OpenOptions,
  io::Write as IoWrite,
  path::{Path, PathBuf},
};

const MAX_TEMPORARY_PATH_ATTEMPTS: usize = 16;

#[derive(Debug)]
pub struct Write {
  authorization_manager: AuthorizationManager,
}

impl Write {
  pub fn new(authorization_manager: AuthorizationManager) -> Self {
    return Self {
      authorization_manager,
    };
  }

  pub fn execute(
    &self,
    task: &TaskRequest,
    request: WriteRequest,
  ) -> Result<WriteOutput, WriteError> {
    if let Err(error) = self.authorization_manager.authorize(
      &task.execution_policy(),
      ExecutionRequirement::new(SandboxMode::WorkspaceWrite, NetworkAccess::Denied),
    ) {
      return Err(WriteError::Authorization(error));
    }
    let path = match resolve_target_path(task.workspace().path(), request.path()) {
      Ok(path) => path,
      Err(error) => return Err(error),
    };
    let temporary_path = match write_temporary_file(&path, request.contents().as_bytes()) {
      Ok(path) => path,
      Err(error) => return Err(error),
    };
    if let Err(source) = std::fs::rename(&temporary_path, &path) {
      let _: std::io::Result<()> = std::fs::remove_file(&temporary_path);

      return Err(WriteError::Replace { path, source });
    }
    let output = WriteOutput::new(path, request.contents().len());

    return Ok(output);
  }
}

#[derive(Debug)]
pub enum WriteError {
  Authorization(AuthorizationError),
  MissingFileName(PathBuf),
  CanonicalizeParent {
    path: PathBuf,
    source: std::io::Error,
  },
  OutsideWorkspace {
    path: PathBuf,
  },
  CanonicalizeExisting {
    path: PathBuf,
    source: std::io::Error,
  },
  Metadata {
    path: PathBuf,
    source: std::io::Error,
  },
  NotFile(PathBuf),
  CreateTemporary {
    path: PathBuf,
    source: std::io::Error,
  },
  WriteTemporary {
    path: PathBuf,
    source: std::io::Error,
  },
  SyncTemporary {
    path: PathBuf,
    source: std::io::Error,
  },
  Replace {
    path: PathBuf,
    source: std::io::Error,
  },
  TemporaryPath,
}

fn resolve_target_path(workspace: &Path, requested_path: &Path) -> Result<PathBuf, WriteError> {
  let candidate_path = if requested_path.is_absolute() {
    requested_path.to_path_buf()
  } else {
    workspace.join(requested_path)
  };
  let file_name = match candidate_path.file_name() {
    Some(file_name) => file_name,
    None => return Err(WriteError::MissingFileName(candidate_path)),
  };
  let parent = match candidate_path.parent() {
    Some(parent) => parent,
    None => return Err(WriteError::MissingFileName(candidate_path)),
  };
  let canonical_parent = match std::fs::canonicalize(parent) {
    Ok(path) => path,
    Err(source) => {
      return Err(WriteError::CanonicalizeParent {
        path: parent.to_path_buf(),
        source,
      });
    }
  };
  if !canonical_parent.starts_with(workspace) {
    return Err(WriteError::OutsideWorkspace {
      path: canonical_parent,
    });
  }
  let path = canonical_parent.join(file_name);

  if path.exists() {
    let canonical_path = match std::fs::canonicalize(&path) {
      Ok(path) => path,
      Err(source) => {
        return Err(WriteError::CanonicalizeExisting { path, source });
      }
    };
    if !canonical_path.starts_with(workspace) {
      return Err(WriteError::OutsideWorkspace {
        path: canonical_path,
      });
    }
    let metadata = match std::fs::metadata(&path) {
      Ok(metadata) => metadata,
      Err(source) => return Err(WriteError::Metadata { path, source }),
    };
    if !metadata.is_file() {
      return Err(WriteError::NotFile(path));
    }
  }

  return Ok(path);
}

fn write_temporary_file(path: &Path, contents: &[u8]) -> Result<PathBuf, WriteError> {
  let parent = match path.parent() {
    Some(parent) => parent,
    None => return Err(WriteError::MissingFileName(path.to_path_buf())),
  };

  for _ in 0..MAX_TEMPORARY_PATH_ATTEMPTS {
    let id = match IdGenerator::generate("write") {
      Ok(id) => id,
      Err(_) => return Err(WriteError::TemporaryPath),
    };
    let temporary_path = parent.join(format!(".{id}.tmp"));
    let mut file = match OpenOptions::new()
      .create_new(true)
      .write(true)
      .open(&temporary_path)
    {
      Ok(file) => file,
      Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => continue,
      Err(source) => {
        return Err(WriteError::CreateTemporary {
          path: temporary_path,
          source,
        });
      }
    };
    if let Err(source) = file.write_all(contents) {
      drop(file);
      let _: std::io::Result<()> = std::fs::remove_file(&temporary_path);

      return Err(WriteError::WriteTemporary {
        path: temporary_path,
        source,
      });
    }
    if let Err(source) = file.sync_all() {
      drop(file);
      let _: std::io::Result<()> = std::fs::remove_file(&temporary_path);

      return Err(WriteError::SyncTemporary {
        path: temporary_path,
        source,
      });
    }
    drop(file);

    return Ok(temporary_path);
  }

  return Err(WriteError::TemporaryPath);
}
