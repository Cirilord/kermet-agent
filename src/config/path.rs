use std::{
  io::Write,
  path::{Path, PathBuf},
};

use super::ConfigError;

const CONFIGURATION_DIRECTORY: &str = ".kermet";
const CONFIGURATION_FILE: &str = "config.toml";
const INITIAL_CONFIGURATION: &str =
  "# Kermet configuration. Credentials are stored in the system keyring.\n\n";

pub(super) fn initial_contents() -> &'static str {
  return INITIAL_CONFIGURATION;
}

pub(crate) fn ensure_exists() -> Result<(), ConfigError> {
  let path = match user_path() {
    Ok(path) => path,
    Err(error) => return Err(error),
  };
  let result = ensure_exists_at(&path);

  return result;
}

pub(crate) fn read() -> Result<String, ConfigError> {
  let path = match user_path() {
    Ok(path) => path,
    Err(error) => return Err(error),
  };
  let result = read_at(&path);

  return result;
}

pub(super) fn exists_at(path: &Path) -> Result<bool, ConfigError> {
  let metadata = match std::fs::metadata(path) {
    Ok(metadata) => metadata,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
    Err(_) => return Err(ConfigError::Unreadable),
  };

  return Ok(metadata.is_file());
}

pub(super) fn ensure_exists_at(path: &Path) -> Result<(), ConfigError> {
  let exists = match exists_at(path) {
    Ok(exists) => exists,
    Err(error) => return Err(error),
  };

  if exists {
    return Ok(());
  }

  let directory = match path.parent() {
    Some(directory) => directory,
    None => return Err(ConfigError::InitializationFailed),
  };

  if std::fs::create_dir_all(directory).is_err() {
    return Err(ConfigError::InitializationFailed);
  }

  match std::fs::OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(path)
  {
    Ok(mut file) => {
      if file.write_all(initial_contents().as_bytes()).is_err() {
        return Err(ConfigError::InitializationFailed);
      }

      return Ok(());
    }
    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
      let exists = match exists_at(path) {
        Ok(exists) => exists,
        Err(error) => return Err(error),
      };

      if exists {
        return Ok(());
      }

      return Err(ConfigError::InitializationFailed);
    }
    Err(_) => return Err(ConfigError::InitializationFailed),
  }
}

pub(super) fn read_at(path: &Path) -> Result<String, ConfigError> {
  let contents = match std::fs::read_to_string(path) {
    Ok(contents) => contents,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
      return Err(ConfigError::NotFound);
    }
    Err(_) => return Err(ConfigError::Unreadable),
  };

  return Ok(contents);
}

pub(super) fn user_path() -> Result<PathBuf, ConfigError> {
  let home = match std::env::var_os("HOME") {
    Some(home) => PathBuf::from(home),
    None => return Err(ConfigError::HomeUnavailable),
  };
  let path = home.join(CONFIGURATION_DIRECTORY).join(CONFIGURATION_FILE);

  return Ok(path);
}
