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
  #[arg(long, value_enum, default_value_t = PermissionMode::ReadOnly)]
  pub mode: PermissionMode,
}

#[derive(Debug, Subcommand)]
pub enum Command {
  /// Inspect the current workspace and local prerequisites.
  Doctor(DoctorArgs),
  /// Start a task in the selected workspace.
  Run(RunArgs),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum PermissionMode {
  ReadOnly,
  WorkspaceWrite,
}

impl std::fmt::Display for PermissionMode {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let value = match *self {
      Self::ReadOnly => "read-only",
      Self::WorkspaceWrite => "workspace-write",
    };

    return formatter.write_str(value);
  }
}

#[cfg(test)]
mod tests {
  use super::{Cli, Command, PermissionMode};
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
    assert_eq!(arguments.mode, PermissionMode::WorkspaceWrite);

    return;
  }
}
