use super::{EditOutput, EditRequest, request::MAX_CONTENT_BYTES};
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
pub struct Edit {
  authorization_manager: AuthorizationManager,
}

impl Edit {
  pub fn new(authorization_manager: AuthorizationManager) -> Self {
    return Self {
      authorization_manager,
    };
  }

  pub fn execute(&self, task: &TaskRequest, request: EditRequest) -> Result<EditOutput, EditError> {
    if let Err(error) = self.authorization_manager.authorize(
      &task.execution_policy(),
      ExecutionRequirement::new(SandboxMode::WorkspaceWrite, NetworkAccess::Denied),
    ) {
      return Err(EditError::Authorization(error));
    }
    let path = match resolve_target_path(task.workspace().path(), request.path()) {
      Ok(path) => path,
      Err(error) => return Err(error),
    };
    let contents = match std::fs::read_to_string(&path) {
      Ok(contents) => contents,
      Err(source) if source.kind() == std::io::ErrorKind::InvalidData => {
        return Err(EditError::NotUtf8 { path });
      }
      Err(source) => return Err(EditError::Read { path, source }),
    };
    let occurrence = match find_unique_occurrence(&contents, request.old_text()) {
      Ok(occurrence) => occurrence,
      Err(error) => return Err(error),
    };
    let updated_contents = format!(
      "{}{}{}",
      &contents[..occurrence],
      request.new_text(),
      &contents[occurrence + request.old_text().len()..]
    );
    if updated_contents.len() > MAX_CONTENT_BYTES {
      return Err(EditError::ResultTooLarge {
        limit: MAX_CONTENT_BYTES,
      });
    }
    let temporary_path = match write_temporary_file(&path, updated_contents.as_bytes()) {
      Ok(path) => path,
      Err(error) => return Err(error),
    };
    if let Err(source) = std::fs::rename(&temporary_path, &path) {
      let _: std::io::Result<()> = std::fs::remove_file(&temporary_path);

      return Err(EditError::Replace { path, source });
    }
    let output = EditOutput::new(path, updated_contents.len());

    return Ok(output);
  }
}

#[derive(Debug)]
pub enum EditError {
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
  Read {
    path: PathBuf,
    source: std::io::Error,
  },
  NotUtf8 {
    path: PathBuf,
  },
  OldTextNotFound,
  OldTextAmbiguous,
  ResultTooLarge {
    limit: usize,
  },
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

fn resolve_target_path(workspace: &Path, requested_path: &Path) -> Result<PathBuf, EditError> {
  let candidate_path = if requested_path.is_absolute() {
    requested_path.to_path_buf()
  } else {
    workspace.join(requested_path)
  };
  let file_name = match candidate_path.file_name() {
    Some(file_name) => file_name,
    None => return Err(EditError::MissingFileName(candidate_path)),
  };
  let parent = match candidate_path.parent() {
    Some(parent) => parent,
    None => return Err(EditError::MissingFileName(candidate_path)),
  };
  let canonical_parent = match std::fs::canonicalize(parent) {
    Ok(path) => path,
    Err(source) => {
      return Err(EditError::CanonicalizeParent {
        path: parent.to_path_buf(),
        source,
      });
    }
  };
  if !canonical_parent.starts_with(workspace) {
    return Err(EditError::OutsideWorkspace {
      path: canonical_parent,
    });
  }
  let path = canonical_parent.join(file_name);
  let canonical_path = match std::fs::canonicalize(&path) {
    Ok(path) => path,
    Err(source) => {
      return Err(EditError::CanonicalizeExisting { path, source });
    }
  };
  if !canonical_path.starts_with(workspace) {
    return Err(EditError::OutsideWorkspace {
      path: canonical_path,
    });
  }
  let metadata = match std::fs::metadata(&path) {
    Ok(metadata) => metadata,
    Err(source) => return Err(EditError::Metadata { path, source }),
  };
  if !metadata.is_file() {
    return Err(EditError::NotFile(path));
  }

  return Ok(path);
}

fn find_unique_occurrence(contents: &str, old_text: &str) -> Result<usize, EditError> {
  let first = match contents.find(old_text) {
    Some(index) => index,
    None => return Err(EditError::OldTextNotFound),
  };
  let first_character_length = match contents[first..].chars().next() {
    Some(character) => character.len_utf8(),
    None => return Err(EditError::OldTextNotFound),
  };
  let search_start = first + first_character_length;

  if contents[search_start..].contains(old_text) {
    return Err(EditError::OldTextAmbiguous);
  }

  return Ok(first);
}

fn write_temporary_file(path: &Path, contents: &[u8]) -> Result<PathBuf, EditError> {
  let parent = match path.parent() {
    Some(parent) => parent,
    None => return Err(EditError::MissingFileName(path.to_path_buf())),
  };

  for _ in 0..MAX_TEMPORARY_PATH_ATTEMPTS {
    let id = match IdGenerator::generate("edit") {
      Ok(id) => id,
      Err(_) => return Err(EditError::TemporaryPath),
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
        return Err(EditError::CreateTemporary {
          path: temporary_path,
          source,
        });
      }
    };
    if let Err(source) = file.write_all(contents) {
      drop(file);
      let _: std::io::Result<()> = std::fs::remove_file(&temporary_path);

      return Err(EditError::WriteTemporary {
        path: temporary_path,
        source,
      });
    }
    if let Err(source) = file.sync_all() {
      drop(file);
      let _: std::io::Result<()> = std::fs::remove_file(&temporary_path);

      return Err(EditError::SyncTemporary {
        path: temporary_path,
        source,
      });
    }
    drop(file);

    return Ok(temporary_path);
  }

  return Err(EditError::TemporaryPath);
}
