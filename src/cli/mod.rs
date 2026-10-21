mod args;
mod workspace;

pub use args::Cli;

use crate::agent::{TaskRequest, TaskRequestError};
use crate::providers::codex::auth::{
  AuthorizationRequest, CodexCredentialStore, TokenClient, UreqTokenTransport,
  receive_authorization_code,
};
use crate::security::KeyringCredentialStore;
use crate::utils::CallbackServer;
use args::{Command, DoctorArgs, LoginArgs, LoginCommand, LogoutArgs, RunArgs};
use clap::CommandFactory;
use std::process::ExitCode;
use workspace::{WorkspaceError, open_workspace};

const INVALID_INPUT_EXIT_CODE: u8 = 2;
const CONFIGURATION_EXIT_CODE: u8 = 78;
const AUTHENTICATION_EXIT_CODE: u8 = 77;

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

fn login(arguments: LoginArgs) -> ExitCode {
  if matches!(arguments.command, Some(LoginCommand::Status)) {
    let exit_code = login_status(arguments.profile);

    return exit_code;
  }

  let credentials =
    match CodexCredentialStore::new(KeyringCredentialStore::new(), arguments.profile) {
      Ok(credentials) => credentials,
      Err(_) => {
        eprintln!("error: secure credential storage is unavailable");
        return ExitCode::from(AUTHENTICATION_EXIT_CODE);
      }
    };
  let server = match CallbackServer::bind() {
    Ok(server) => server,
    Err(_) => {
      eprintln!("error: failed to start the OAuth callback listener");
      return ExitCode::from(AUTHENTICATION_EXIT_CODE);
    }
  };
  let request = AuthorizationRequest::new();

  println!("Open this URL in your browser to authenticate:");
  println!("{}", request.authorization_url());

  let code = match receive_authorization_code(&server, &request) {
    Ok(code) => code,
    Err(_) => {
      eprintln!("error: OAuth callback validation failed");
      return ExitCode::from(AUTHENTICATION_EXIT_CODE);
    }
  };
  let client = TokenClient::new(UreqTokenTransport::new());
  let tokens =
    match client.exchange_authorization_code(&code, &request, std::time::SystemTime::now()) {
      Ok(tokens) => tokens,
      Err(_) => {
        eprintln!("error: failed to exchange the OAuth authorization code");
        return ExitCode::from(AUTHENTICATION_EXIT_CODE);
      }
    };

  if credentials.save(&tokens).is_err() {
    eprintln!("error: failed to save credentials in secure storage");
    return ExitCode::from(AUTHENTICATION_EXIT_CODE);
  }

  println!("Codex authentication completed.");

  return ExitCode::SUCCESS;
}

fn login_status(profile: String) -> ExitCode {
  let credentials = match CodexCredentialStore::new(KeyringCredentialStore::new(), profile) {
    Ok(credentials) => credentials,
    Err(_) => {
      eprintln!("error: secure credential storage is unavailable");
      return ExitCode::from(AUTHENTICATION_EXIT_CODE);
    }
  };
  let tokens = match credentials.load() {
    Ok(tokens) => tokens,
    Err(_) => {
      eprintln!("error: failed to read credentials from secure storage");
      return ExitCode::from(AUTHENTICATION_EXIT_CODE);
    }
  };

  if tokens.is_none() {
    eprintln!("error: Codex is not authenticated");
    return ExitCode::from(AUTHENTICATION_EXIT_CODE);
  }

  println!("Codex authentication is configured.");

  return ExitCode::SUCCESS;
}

fn logout(arguments: LogoutArgs) -> ExitCode {
  let credentials =
    match CodexCredentialStore::new(KeyringCredentialStore::new(), arguments.profile) {
      Ok(credentials) => credentials,
      Err(_) => {
        eprintln!("error: secure credential storage is unavailable");
        return ExitCode::from(AUTHENTICATION_EXIT_CODE);
      }
    };

  if credentials.delete().is_err() {
    eprintln!("error: failed to remove credentials from secure storage");
    return ExitCode::from(AUTHENTICATION_EXIT_CODE);
  }

  println!("Codex credentials were removed.");

  return ExitCode::SUCCESS;
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
    Some(Command::Login(arguments)) => login(arguments),
    Some(Command::Logout(arguments)) => logout(arguments),
    None => print_help(),
  };

  return exit_code;
}
