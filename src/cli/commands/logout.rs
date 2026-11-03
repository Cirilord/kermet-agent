use super::super::{CliAction, CliError};

pub(in super::super) fn logout() -> Result<CliAction, CliError> {
  let action = CliAction::Logout;

  return Ok(action);
}
