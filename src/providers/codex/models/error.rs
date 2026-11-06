use std::fmt::{Display, Formatter};

pub(crate) enum CodexModelsError {
  NotSignedIn,
  Request,
}

impl Display for CodexModelsError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::NotSignedIn => formatter.write_str("no Codex session is available"),
      Self::Request => formatter.write_str("could not retrieve the Codex model catalog"),
    };

    return result;
  }
}
