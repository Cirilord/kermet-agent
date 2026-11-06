use clap::{Args, Subcommand};

use super::super::{CliAction, CliError};

#[derive(Args)]
pub(in super::super) struct ConfigArgs {
  #[command(subcommand)]
  command: ConfigCommand,
}

#[derive(Subcommand)]
enum ConfigCommand {
  /// Print the configuration file.
  Show,
}

pub(in super::super) fn config(args: ConfigArgs) -> Result<CliAction, CliError> {
  let action = match args.command {
    ConfigCommand::Show => CliAction::ShowConfig,
  };

  return Ok(action);
}
