use crate::agent::TaskRequest;
use crate::security::{Capability, PermissionDenied, PermissionManager};
use std::{
  fs::File,
  io::Read,
  path::{Path, PathBuf},
};

const MAX_FILE_BYTES: u64 = 256 * 1024;

#[derive(Debug)]
pub enum ReadFileError {
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

fn resolve_path(workspace: &Path, requested_path: PathBuf) -> Result<PathBuf, ReadFileError> {
  let candidate_path = if requested_path.is_absolute() {
    requested_path
  } else {
    workspace.join(requested_path)
  };
  let path = match std::fs::canonicalize(&candidate_path) {
    Ok(path) => path,
    Err(source) => {
      return Err(ReadFileError::Canonicalize {
        path: candidate_path,
        source,
      });
    }
  };

  if !path.starts_with(workspace) {
    return Err(ReadFileError::OutsideWorkspace { path });
  }

  return Ok(path);
}

#[derive(Debug)]
pub struct ReadFile {
  permission_manager: PermissionManager,
}

impl ReadFile {
  pub fn new(permission_manager: PermissionManager) -> Self {
    return Self { permission_manager };
  }

  pub fn execute(
    &self,
    task: &TaskRequest,
    request: ReadFileRequest,
  ) -> Result<ReadFileOutput, ReadFileError> {
    if let Err(error) = self
      .permission_manager
      .authorize(task.permission_mode(), Capability::ReadFile)
    {
      return Err(ReadFileError::PermissionDenied(error));
    }

    let path = match resolve_path(task.workspace().path(), request.path) {
      Ok(path) => path,
      Err(error) => return Err(error),
    };
    let metadata = match std::fs::metadata(&path) {
      Ok(metadata) => metadata,
      Err(source) => return Err(ReadFileError::Metadata { path, source }),
    };

    if !metadata.is_file() {
      return Err(ReadFileError::NotFile(path));
    }

    if metadata.len() > MAX_FILE_BYTES {
      return Err(ReadFileError::TooLarge {
        path,
        limit: MAX_FILE_BYTES,
      });
    }

    let file = match File::open(&path) {
      Ok(file) => file,
      Err(source) => return Err(ReadFileError::Open { path, source }),
    };
    let mut contents = Vec::new();
    let limit = match MAX_FILE_BYTES.checked_add(1) {
      Some(limit) => limit,
      None => return Err(ReadFileError::LimitOverflow),
    };
    let bytes_read = match file.take(limit).read_to_end(&mut contents) {
      Ok(bytes_read) => bytes_read,
      Err(source) => return Err(ReadFileError::Read { path, source }),
    };

    if bytes_read as u64 > MAX_FILE_BYTES {
      return Err(ReadFileError::TooLarge {
        path,
        limit: MAX_FILE_BYTES,
      });
    }

    let contents = select_lines(contents, request.line_range);

    return Ok(ReadFileOutput { path, contents });
  }
}

#[derive(Debug)]
pub struct ReadFileRequest {
  path: PathBuf,
  line_range: Option<LineRange>,
}

impl ReadFileRequest {
  pub fn new(path: PathBuf) -> Self {
    return Self {
      path,
      line_range: None,
    };
  }

  pub fn with_line_range(
    path: PathBuf,
    start: usize,
    end: usize,
  ) -> Result<Self, ReadFileRequestError> {
    let line_range = match LineRange::new(start, end) {
      Ok(line_range) => line_range,
      Err(error) => return Err(error),
    };

    return Ok(Self {
      path,
      line_range: Some(line_range),
    });
  }
}

#[derive(Clone, Copy, Debug)]
struct LineRange {
  start: usize,
  end: usize,
}

impl LineRange {
  fn new(start: usize, end: usize) -> Result<Self, ReadFileRequestError> {
    if start == 0 || end < start {
      return Err(ReadFileRequestError::InvalidLineRange { start, end });
    }

    return Ok(Self { start, end });
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ReadFileRequestError {
  InvalidLineRange { start: usize, end: usize },
}

fn select_lines(contents: Vec<u8>, line_range: Option<LineRange>) -> Vec<u8> {
  let Some(line_range) = line_range else {
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

    if line_number >= line_range.start && line_number <= line_range.end {
      selected_contents.extend_from_slice(line);
    }
  }

  return selected_contents;
}

#[derive(Debug)]
pub struct ReadFileOutput {
  path: PathBuf,
  contents: Vec<u8>,
}

impl ReadFileOutput {
  pub fn contents(&self) -> &[u8] {
    return &self.contents;
  }

  pub fn path(&self) -> &Path {
    return &self.path;
  }
}

#[cfg(test)]
mod tests {
  use super::{ReadFile, ReadFileError, ReadFileRequest};
  use crate::{
    agent::{PermissionMode, TaskRequest},
    security::PermissionManager,
    workspace::Workspace,
  };
  use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
  };

  #[test]
  fn reads_a_file_inside_the_workspace() -> () {
    let workspace_path = unique_path("workspace");
    fs::create_dir(&workspace_path).expect("the workspace should be created");
    fs::write(workspace_path.join("notes.txt"), "Kermet").expect("the file should be written");
    let workspace = Workspace::new(
      fs::canonicalize(&workspace_path).expect("the path should be canonical"),
      false,
    );
    let task = TaskRequest::new("read notes".to_owned(), workspace, PermissionMode::ReadOnly)
      .expect("the task should be valid");
    let tool = ReadFile::new(PermissionManager::new());

    let output = tool
      .execute(&task, ReadFileRequest::new(PathBuf::from("notes.txt")))
      .expect("the file should be read");

    fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");
    assert_eq!(output.contents(), b"Kermet");

    return;
  }

  #[test]
  fn rejects_a_path_outside_the_workspace() -> () {
    let root = unique_path("root");
    let workspace_path = root.join("workspace");
    fs::create_dir_all(&workspace_path).expect("the workspace should be created");
    fs::write(root.join("outside.txt"), "outside").expect("the file should be written");
    let workspace = Workspace::new(
      fs::canonicalize(&workspace_path).expect("the path should be canonical"),
      false,
    );
    let task = TaskRequest::new("read file".to_owned(), workspace, PermissionMode::ReadOnly)
      .expect("the task should be valid");
    let tool = ReadFile::new(PermissionManager::new());

    let error = tool
      .execute(&task, ReadFileRequest::new(PathBuf::from("../outside.txt")))
      .expect_err("the path should be rejected");

    fs::remove_dir_all(&root).expect("the fixture should be removed");
    assert!(matches!(error, ReadFileError::OutsideWorkspace { .. }));

    return;
  }

  #[test]
  fn reads_only_the_requested_lines() -> () {
    let workspace_path = unique_path("lines");
    fs::create_dir(&workspace_path).expect("the workspace should be created");
    fs::write(workspace_path.join("notes.txt"), "one\ntwo\nthree\n")
      .expect("the file should be written");
    let workspace = Workspace::new(
      fs::canonicalize(&workspace_path).expect("the path should be canonical"),
      false,
    );
    let task = TaskRequest::new("read notes".to_owned(), workspace, PermissionMode::ReadOnly)
      .expect("the task should be valid");
    let request = ReadFileRequest::with_line_range(PathBuf::from("notes.txt"), 2, 3)
      .expect("the range should be valid");
    let output = ReadFile::new(PermissionManager::new())
      .execute(&task, request)
      .expect("the file should be read");

    fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");
    assert_eq!(output.contents(), b"two\nthree\n");

    return;
  }

  #[test]
  fn rejects_a_missing_file() -> () {
    let workspace_path = unique_path("missing");
    fs::create_dir(&workspace_path).expect("the workspace should be created");
    let workspace = Workspace::new(
      fs::canonicalize(&workspace_path).expect("the path should be canonical"),
      false,
    );
    let task = TaskRequest::new("read file".to_owned(), workspace, PermissionMode::ReadOnly)
      .expect("the task should be valid");
    let error = ReadFile::new(PermissionManager::new())
      .execute(&task, ReadFileRequest::new(PathBuf::from("missing.txt")))
      .expect_err("the file should be missing");

    fs::remove_dir_all(&workspace_path).expect("the workspace should be removed");
    assert!(matches!(error, ReadFileError::Canonicalize { .. }));

    return;
  }

  #[cfg(unix)]
  #[test]
  fn rejects_a_symlink_that_escapes_the_workspace() -> () {
    use std::os::unix::fs::symlink;

    let root = unique_path("symlink");
    let workspace_path = root.join("workspace");
    let outside_path = root.join("outside.txt");
    fs::create_dir_all(&workspace_path).expect("the workspace should be created");
    fs::write(&outside_path, "outside").expect("the file should be written");
    symlink(&outside_path, workspace_path.join("escape.txt"))
      .expect("the symlink should be created");
    let workspace = Workspace::new(
      fs::canonicalize(&workspace_path).expect("the path should be canonical"),
      false,
    );
    let task = TaskRequest::new("read file".to_owned(), workspace, PermissionMode::ReadOnly)
      .expect("the task should be valid");
    let error = ReadFile::new(PermissionManager::new())
      .execute(&task, ReadFileRequest::new(PathBuf::from("escape.txt")))
      .expect_err("the symlink should be rejected");

    fs::remove_dir_all(&root).expect("the fixture should be removed");
    assert!(matches!(error, ReadFileError::OutsideWorkspace { .. }));

    return;
  }

  fn unique_path(kind: &str) -> PathBuf {
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("the clock should be valid")
      .as_nanos();

    return std::env::temp_dir().join(format!(
      "kermet-read-file-{kind}-{}-{timestamp}",
      std::process::id()
    ));
  }
}
