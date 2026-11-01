use std::fmt::{Display, Formatter};

pub(crate) enum CliError {
  EmptyLoginProvider,
}

impl Display for CliError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::EmptyLoginProvider => formatter.write_str("provider must not be empty"),
    };

    return result;
  }
}
