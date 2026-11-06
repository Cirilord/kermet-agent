use clap::{Args, Subcommand};

use super::super::{CliAction, CliError};

#[derive(Args)]
pub(in super::super) struct ConfigArgs {
  #[command(subcommand)]
  command: ConfigCommand,
}

#[derive(Subcommand)]
enum ConfigCommand {
  /// Print the value of a configuration key.
  Get(ConfigKeyArgs),
  /// Set the value of a configuration key.
  Set(ConfigSetArgs),
  /// Print the configuration file.
  Show,
  /// Remove a configuration key.
  Unset(ConfigKeyArgs),
}

#[derive(Args)]
struct ConfigKeyArgs {
  key: String,
}

#[derive(Args)]
struct ConfigSetArgs {
  key: String,
  value: String,
}

pub(crate) struct ConfigKeyInput {
  key: String,
}

impl ConfigKeyInput {
  pub(crate) fn key(&self) -> &str {
    return &self.key;
  }
}

pub(crate) struct ConfigSetInput {
  key: String,
  value: String,
}

impl ConfigSetInput {
  pub(crate) fn key(&self) -> &str {
    return &self.key;
  }

  pub(crate) fn value(&self) -> &str {
    return &self.value;
  }
}

pub(in super::super) fn config(args: ConfigArgs) -> Result<CliAction, CliError> {
  let action = match args.command {
    ConfigCommand::Get(args) => CliAction::GetConfig(ConfigKeyInput { key: args.key }),
    ConfigCommand::Set(args) => CliAction::SetConfig(ConfigSetInput {
      key: args.key,
      value: args.value,
    }),
    ConfigCommand::Show => CliAction::ShowConfig,
    ConfigCommand::Unset(args) => CliAction::UnsetConfig(ConfigKeyInput { key: args.key }),
  };

  return Ok(action);
}
