use std::fmt::{Display, Formatter};

pub(crate) enum CliError {
  EmptyProvider,
}

impl Display for CliError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::EmptyProvider => formatter.write_str("provider must not be empty"),
    };

    return result;
  }
}
