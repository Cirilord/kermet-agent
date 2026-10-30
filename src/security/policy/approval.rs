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
}

impl Display for ApprovalPolicy {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = formatter.write_str(self.identifier());

    return result;
  }
}
