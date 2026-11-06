use super::super::{CliAction, CliError};

pub(in super::super) fn list_models() -> Result<CliAction, CliError> {
  let action = CliAction::ListModels;

  return Ok(action);
}
