use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalPolicy {
  OnRequest,
  Never,
}

impl ApprovalPolicy {
  pub fn identifier(&self) -> &'static str {
    let identifier = match *self {
      Self::OnRequest => "on-request",
      Self::Never => "never",
    };

    return identifier;
  }

  pub fn from_identifier(value: &str) -> Option<Self> {
    let policy = match value {
      "on-request" => Some(Self::OnRequest),
      "never" => Some(Self::Never),
      _ => None,
    };

    return policy;
  }
}

impl Display for ApprovalPolicy {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = formatter.write_str(self.identifier());

    return result;
  }
}
