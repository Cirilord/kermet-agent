use clap::{Parser, Subcommand};

use super::{
  CliError,
  action::CliAction,
  commands::{self, LoginArgs},
};

#[derive(Parser)]
#[command(name = "kermet", version, about = "A local coding agent")]
pub(super) struct Cli {
  #[command(subcommand)]
  pub(super) command: Command,
}

#[derive(Subcommand)]
pub(super) enum Command {
  /// Authenticate with a provider.
  Login(LoginArgs),
  /// List the models available to the active provider.
  Models,
  /// Sign out from the active provider.
  Logout,
}

pub(crate) fn parse() -> Result<CliAction, CliError> {
  let cli = Cli::parse();
  let result = match cli.command {
    Command::Login(args) => commands::login(args),
    Command::Models => commands::list_models(),
    Command::Logout => commands::logout(),
  };

  return result;
}
