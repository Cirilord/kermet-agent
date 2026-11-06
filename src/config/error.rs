use std::fmt::{Display, Formatter};

pub(crate) enum ConfigError {
  HomeUnavailable,
  InitializationFailed,
  NotFound,
  Unreadable,
}

impl Display for ConfigError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::HomeUnavailable => formatter.write_str("could not determine the home directory"),
      Self::InitializationFailed => {
        formatter.write_str("could not initialize the configuration file")
      }
      Self::NotFound => formatter.write_str("the configuration file does not exist"),
      Self::Unreadable => formatter.write_str("could not access the configuration file"),
    };

    return result;
  }
}
