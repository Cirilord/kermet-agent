use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
  ReadFile,
  SearchFiles,
  WriteFile,
  ExecuteProcess,
}

impl Capability {
  pub fn identifier(&self) -> &'static str {
    let identifier = match *self {
      Self::ReadFile => "read-file",
      Self::SearchFiles => "search-files",
      Self::WriteFile => "write-file",
      Self::ExecuteProcess => "execute-process",
    };

    return identifier;
  }
}

impl Display for Capability {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = formatter.write_str(self.identifier());

    return result;
  }
}

#[cfg(test)]
mod tests {
  use super::Capability;

  #[test]
  fn exposes_stable_identifiers() -> () {
    assert_eq!(Capability::ReadFile.identifier(), "read-file");
    assert_eq!(Capability::SearchFiles.identifier(), "search-files");
    assert_eq!(Capability::WriteFile.identifier(), "write-file");
    assert_eq!(Capability::ExecuteProcess.identifier(), "execute-process");

    return;
  }
}
