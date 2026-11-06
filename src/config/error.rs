use std::fmt::{Display, Formatter};

pub(crate) enum ConfigError {
  HomeUnavailable,
  InitializationFailed,
  InvalidDocument,
  InvalidKey,
  KeyNotFound,
  NotFound,
  PathNotTable,
  Unreadable,
  UnsupportedValue,
  WriteFailed,
}

impl Display for ConfigError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::HomeUnavailable => formatter.write_str("could not determine the home directory"),
      Self::InitializationFailed => {
        formatter.write_str("could not initialize the configuration file")
      }
      Self::InvalidDocument => formatter.write_str("the configuration file is not valid TOML"),
      Self::InvalidKey => formatter.write_str("the configuration key is invalid"),
      Self::KeyNotFound => formatter.write_str("the configuration key does not exist"),
      Self::NotFound => formatter.write_str("the configuration file does not exist"),
      Self::PathNotTable => formatter.write_str("the configuration path does not refer to a table"),
      Self::Unreadable => formatter.write_str("could not access the configuration file"),
      Self::UnsupportedValue => formatter.write_str("the configuration value is not a string"),
      Self::WriteFailed => formatter.write_str("could not update the configuration file"),
    };

    return result;
  }
}
