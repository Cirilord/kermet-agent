use super::{ReadOutput, ReadRequest};
use crate::{
  agent::TaskRequest,
  security::{Capability, PermissionDenied, PermissionManager},
};
use std::{
  fs::File,
  io::Read as IoRead,
  path::{Path, PathBuf},
};

const MAX_FILE_BYTES: u64 = 256 * 1024;

#[derive(Debug)]
pub struct Read {
  permission_manager: PermissionManager,
}

impl Read {
  pub fn new(permission_manager: PermissionManager) -> Self {
    return Self { permission_manager };
  }

  pub fn execute(&self, task: &TaskRequest, request: ReadRequest) -> Result<ReadOutput, ReadError> {
    if let Err(error) = self
      .permission_manager
      .authorize(task.permission_mode(), Capability::ReadFile)
    {
      return Err(ReadError::PermissionDenied(error));
    }

    let path = match resolve_path(task.workspace().path(), request.path()) {
      Ok(path) => path,
      Err(error) => return Err(error),
    };
    let metadata = match std::fs::metadata(&path) {
      Ok(metadata) => metadata,
      Err(source) => return Err(ReadError::Metadata { path, source }),
    };
    if !metadata.is_file() {
      return Err(ReadError::NotFile(path));
    }
    if metadata.len() > MAX_FILE_BYTES {
      return Err(ReadError::TooLarge {
        path,
        limit: MAX_FILE_BYTES,
      });
    }
    let file = match File::open(&path) {
      Ok(file) => file,
      Err(source) => return Err(ReadError::Open { path, source }),
    };
    let mut contents = Vec::new();
    let limit = match MAX_FILE_BYTES.checked_add(1) {
      Some(limit) => limit,
      None => return Err(ReadError::LimitOverflow),
    };
    let bytes_read = match file.take(limit).read_to_end(&mut contents) {
      Ok(bytes_read) => bytes_read,
      Err(source) => return Err(ReadError::Read { path, source }),
    };
    if bytes_read as u64 > MAX_FILE_BYTES {
      return Err(ReadError::TooLarge {
        path,
        limit: MAX_FILE_BYTES,
      });
    }
    let contents = select_lines(contents, request.line_range());

    return Ok(ReadOutput::new(path, contents));
  }
}

#[derive(Debug)]
pub enum ReadError {
  PermissionDenied(PermissionDenied),
  Canonicalize {
    path: PathBuf,
    source: std::io::Error,
  },
  OutsideWorkspace {
    path: PathBuf,
  },
  Metadata {
    path: PathBuf,
    source: std::io::Error,
  },
  NotFile(PathBuf),
  TooLarge {
    path: PathBuf,
    limit: u64,
  },
  Open {
    path: PathBuf,
    source: std::io::Error,
  },
  Read {
    path: PathBuf,
    source: std::io::Error,
  },
  LimitOverflow,
}

fn resolve_path(workspace: &Path, requested_path: &Path) -> Result<PathBuf, ReadError> {
  let candidate_path = if requested_path.is_absolute() {
    requested_path.to_path_buf()
  } else {
    workspace.join(requested_path)
  };
  let path = match std::fs::canonicalize(&candidate_path) {
    Ok(path) => path,
    Err(source) => {
      return Err(ReadError::Canonicalize {
        path: candidate_path,
        source,
      });
    }
  };
  if !path.starts_with(workspace) {
    return Err(ReadError::OutsideWorkspace { path });
  }

  return Ok(path);
}

fn select_lines(contents: Vec<u8>, line_range: Option<(usize, usize)>) -> Vec<u8> {
  let Some((start, end)) = line_range else {
    return contents;
  };
  let mut selected_contents = Vec::new();

  for (index, line) in contents
    .split_inclusive(|byte| {
      return *byte == b'\n';
    })
    .enumerate()
  {
    let line_number = index + 1;
    if line_number >= start && line_number <= end {
      selected_contents.extend_from_slice(line);
    }
  }

  return selected_contents;
}
