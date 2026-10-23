use crate::{
  ids::IdGenerator,
  thread::state::{SCHEMA_VERSION, Thread},
};
use std::{
  fs::{self, File, OpenOptions},
  io::Write,
  path::{Path, PathBuf},
};

const TEMPORARY_FILE_ATTEMPTS: usize = 8;

#[derive(Debug, Eq, PartialEq)]
pub enum ThreadStoreError {
  InvalidId,
  NotFound,
  Unreadable,
  Unwritable,
  InvalidThread,
}

pub fn user_directory(home: &Path) -> PathBuf {
  return home.join(".kermet").join("threads");
}

pub struct ThreadStore {
  directory: PathBuf,
}

impl ThreadStore {
  pub fn new(directory: PathBuf) -> Self {
    return Self { directory };
  }

  pub fn save(&self, thread: &Thread) -> Result<(), ThreadStoreError> {
    if fs::create_dir_all(&self.directory).is_err() {
      return Err(ThreadStoreError::Unwritable);
    }

    let contents = match serde_json::to_vec_pretty(thread) {
      Ok(contents) => contents,
      Err(_) => return Err(ThreadStoreError::InvalidThread),
    };

    let (temporary_path, mut file) = match self.open_temporary_file() {
      Ok(value) => value,
      Err(error) => return Err(error),
    };

    if file.write_all(&contents).is_err() || file.sync_all().is_err() {
      let _: std::io::Result<()> = fs::remove_file(&temporary_path);

      return Err(ThreadStoreError::Unwritable);
    }

    if fs::rename(&temporary_path, self.path(thread.id())).is_err() {
      let _: std::io::Result<()> = fs::remove_file(&temporary_path);

      return Err(ThreadStoreError::Unwritable);
    }

    return Ok(());
  }

  pub fn load(&self, id: &str) -> Result<Thread, ThreadStoreError> {
    if IdGenerator::validate("thread", id).is_err() {
      return Err(ThreadStoreError::InvalidId);
    }

    let contents = match fs::read(self.path(id)) {
      Ok(contents) => contents,
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
        return Err(ThreadStoreError::NotFound);
      }
      Err(_) => return Err(ThreadStoreError::Unreadable),
    };

    let thread = match serde_json::from_slice::<Thread>(&contents) {
      Ok(thread) => thread,
      Err(_) => return Err(ThreadStoreError::InvalidThread),
    };

    if thread.version != SCHEMA_VERSION || thread.id != id {
      return Err(ThreadStoreError::InvalidThread);
    }

    return Ok(thread);
  }

  pub fn list(&self) -> Result<Vec<Thread>, ThreadStoreError> {
    let entries = match fs::read_dir(&self.directory) {
      Ok(entries) => entries,
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
      Err(_) => return Err(ThreadStoreError::Unreadable),
    };
    let mut threads = Vec::new();

    for entry in entries {
      let entry = match entry {
        Ok(entry) => entry,
        Err(_) => return Err(ThreadStoreError::Unreadable),
      };
      let file_type = match entry.file_type() {
        Ok(file_type) => file_type,
        Err(_) => return Err(ThreadStoreError::Unreadable),
      };

      if !file_type.is_file() {
        continue;
      }

      let file_name = entry.file_name();
      let Some(file_name) = file_name.to_str() else {
        return Err(ThreadStoreError::InvalidThread);
      };
      let Some(id) = file_name.strip_suffix(".json") else {
        continue;
      };
      let thread = match self.load(id) {
        Ok(thread) => thread,
        Err(error) => return Err(error),
      };

      threads.push(thread);
    }

    threads.sort_by(|left, right| return left.id().cmp(right.id()));

    return Ok(threads);
  }

  fn path(&self, id: &str) -> PathBuf {
    return self.directory.join(format!("{id}.json"));
  }

  fn open_temporary_file(&self) -> Result<(PathBuf, File), ThreadStoreError> {
    for _ in 0..TEMPORARY_FILE_ATTEMPTS {
      let nonce = rand::random::<u64>();
      let path = self
        .directory
        .join(format!(".thread-{}-{nonce}.tmp", std::process::id()));

      let file = match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
        Err(_) => return Err(ThreadStoreError::Unwritable),
      };

      return Ok((path, file));
    }

    return Err(ThreadStoreError::Unwritable);
  }
}
