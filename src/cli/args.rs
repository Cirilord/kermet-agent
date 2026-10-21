use crate::agent::PermissionMode;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "kermet", version, about = "A local coding agent")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Option<Command>,
}

#[derive(Args, Debug)]
pub struct DoctorArgs {
  /// Workspace to inspect.
  #[arg(long, default_value = ".")]
  pub workspace: PathBuf,
}

#[derive(Args, Debug)]
pub struct RunArgs {
  /// Task for Kermet to perform.
  pub task: String,
  /// Workspace in which the task will run.
  #[arg(long, default_value = ".")]
  pub workspace: PathBuf,
  /// Maximum permission mode available to the task.
  #[arg(long, value_enum, default_value_t = CliPermissionMode::ReadOnly)]
  pub mode: CliPermissionMode,
}

#[derive(Args, Debug)]
pub struct LoginArgs {
  /// Credential profile to authenticate.
  #[arg(long, default_value = "default", global = true)]
  pub profile: String,
  #[command(subcommand)]
  pub command: Option<LoginCommand>,
}

#[derive(Debug, Subcommand)]
pub enum LoginCommand {
  /// Report whether the selected Codex profile is authenticated.
  Status,
}

#[derive(Args, Debug)]
pub struct LogoutArgs {
  /// Credential profile to remove.
  #[arg(long, default_value = "default")]
  pub profile: String,
}

#[derive(Debug, Subcommand)]
pub enum Command {
  /// Inspect the current workspace and local prerequisites.
  Doctor(DoctorArgs),
  /// Start a task in the selected workspace.
  Run(RunArgs),
  /// Authenticate with Codex through the browser OAuth flow.
  Login(LoginArgs),
  /// Remove saved Codex credentials.
  Logout(LogoutArgs),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliPermissionMode {
  ReadOnly,
  WorkspaceWrite,
}

impl From<CliPermissionMode> for PermissionMode {
  fn from(mode: CliPermissionMode) -> Self {
    let permission_mode = match mode {
      CliPermissionMode::ReadOnly => Self::ReadOnly,
      CliPermissionMode::WorkspaceWrite => Self::WorkspaceWrite,
    };

    return permission_mode;
  }
}

#[cfg(test)]
mod tests {
  use super::{Cli, CliPermissionMode, Command, LoginCommand};
  use clap::Parser;

  #[test]
  fn parses_run_arguments() -> () {
    let cli = Cli::try_parse_from([
      "kermet",
      "run",
      "inspect the authentication flow",
      "--workspace",
      "./workspace",
      "--mode",
      "workspace-write",
    ])
    .expect("the command-line arguments should parse");

    let Command::Run(arguments) = cli.command.expect("a subcommand should be present") else {
      panic!("the run subcommand should parse");
    };

    assert_eq!(arguments.task, "inspect the authentication flow");
    assert_eq!(arguments.workspace, std::path::PathBuf::from("./workspace"));
    assert_eq!(arguments.mode, CliPermissionMode::WorkspaceWrite);

    return;
  }

  #[test]
  fn parses_login_status() -> () {
    let cli = Cli::try_parse_from(["kermet", "login", "status", "--profile", "work"])
      .expect("the command-line arguments should parse");
    let Command::Login(arguments) = cli.command.expect("a subcommand should be present") else {
      panic!("the login subcommand should parse");
    };

    assert!(matches!(arguments.command, Some(LoginCommand::Status)));
    assert_eq!(arguments.profile, "work");

    return;
  }
}
