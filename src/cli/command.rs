use clap::{Parser, Subcommand};

use super::{
  CliError,
  action::CliAction,
  commands::{self, ConfigArgs, LoginArgs, ModelsArgs},
};

#[derive(Parser)]
#[command(name = "kermet", version, about = "A local coding agent")]
pub(super) struct Cli {
  #[command(subcommand)]
  pub(super) command: Command,
}

#[derive(Subcommand)]
pub(super) enum Command {
  /// Manage the Kermet configuration.
  Config(ConfigArgs),
  /// Authenticate with a provider.
  Login(LoginArgs),
  /// List models or set the default model profile.
  Models(ModelsArgs),
  /// Sign out from the active provider.
  Logout,
}

pub(crate) fn parse() -> Result<CliAction, CliError> {
  let cli = Cli::parse();
  let result = match cli.command {
    Command::Config(args) => commands::config(args),
    Command::Login(args) => commands::login(args),
    Command::Models(args) => commands::models(args),
    Command::Logout => commands::logout(),
  };

  return result;
}
