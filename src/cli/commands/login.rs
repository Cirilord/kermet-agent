use clap::Args;

use super::super::{action::CliAction, error::CliError};

#[derive(Args)]
pub(in super::super) struct LoginArgs {
  /// Provider to authenticate with.
  provider: String,
}

pub(crate) struct LoginInput {
  provider: String,
}

impl LoginInput {
  pub(crate) fn provider(&self) -> &str {
    return &self.provider;
  }
}

pub(in super::super) fn login(args: LoginArgs) -> Result<CliAction, CliError> {
  let provider = args.provider.trim();

  if provider.is_empty() {
    return Err(CliError::EmptyLoginProvider);
  }
  let input = LoginInput {
    provider: provider.to_owned(),
  };
  let action = CliAction::Login(input);

  return Ok(action);
}
