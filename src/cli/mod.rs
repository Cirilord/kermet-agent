mod args;
mod workspace;

pub use args::Cli;

use crate::agent::{TaskRequest, TaskRequestError};
use args::{Command, DoctorArgs, RunArgs};
use clap::CommandFactory;
use std::process::ExitCode;
use workspace::{WorkspaceError, open_workspace};

const INVALID_INPUT_EXIT_CODE: u8 = 2;
const CONFIGURATION_EXIT_CODE: u8 = 78;

fn report_workspace_error(error: &WorkspaceError) -> () {
  eprintln!("error: {error}");

  return;
}

fn report_task_request_error(error: &TaskRequestError) -> () {
  eprintln!("error: {error}");

  return;
}

fn doctor(arguments: DoctorArgs) -> ExitCode {
  let workspace = match open_workspace(arguments.workspace) {
    Ok(workspace) => workspace,
    Err(error) => {
      report_workspace_error(&error);
      return ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };

  println!("Workspace: {}", workspace.path().display());
  println!(
    "Git metadata: {}",
    if workspace.has_git_metadata() {
      "detected"
    } else {
      "not detected"
    }
  );
  println!("Model provider: not configured");

  return ExitCode::SUCCESS;
}

fn run_task(arguments: RunArgs) -> ExitCode {
  let workspace = match open_workspace(arguments.workspace) {
    Ok(workspace) => workspace,
    Err(error) => {
      report_workspace_error(&error);
      return ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };
  let request = match TaskRequest::new(arguments.task, workspace, arguments.mode.into()) {
    Ok(request) => request,
    Err(error) => {
      report_task_request_error(&error);
      return ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };

  eprintln!("error: no model provider is configured");
  eprintln!("task: {}", request.task());
  eprintln!("workspace: {}", request.workspace().path().display());
  eprintln!("permission mode: {}", request.permission_mode());

  return ExitCode::from(CONFIGURATION_EXIT_CODE);
}

fn print_help() -> ExitCode {
  let mut command = Cli::command();

  if let Err(error) = command.print_help() {
    eprintln!("error: failed to print help: {error}");
    return ExitCode::FAILURE;
  }

  println!();

  return ExitCode::SUCCESS;
}

pub fn execute(cli: Cli) -> ExitCode {
  let exit_code = match cli.command {
    Some(Command::Doctor(arguments)) => doctor(arguments),
    Some(Command::Run(arguments)) => run_task(arguments),
    None => print_help(),
  };

  return exit_code;
}
