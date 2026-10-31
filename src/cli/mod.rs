mod args;
mod workspace;

pub use args::Cli;

use crate::agent::{AgentEvent, AgentLoopError, TaskRequest, TaskRequestError};
use crate::config::{
  ConfigError, ModelOverrides, get, initialize, resolve_model_selection, set, unset, user_path,
};
use crate::hub::{HubError, serve};
use crate::ids::IdGenerator;
use crate::models::{ModelEvent, ModelProviderError};
use crate::providers::ProviderRegistryError;
use crate::providers::codex::auth::{
  AuthorizationRequest, CodexCredentialStore, TokenClient, UreqTokenTransport,
  receive_authorization_code,
};
use crate::security::{ExecutionPolicy, ExecutionPolicyError, KeyringCredentialStore};
use crate::thread::{Thread, ThreadStore, ThreadStoreError};
use crate::turn::{TurnError, execute_turn};
use crate::utils::CallbackServer;
use args::{
  Command, ConfigArgs, ConfigCommand, DoctorArgs, HubArgs, LoginArgs, LoginCommand, LogoutArgs,
  RunArgs,
};
use clap::CommandFactory;
use rfd::FileDialog;
use std::{
  io::{self, Write},
  process::ExitCode,
};
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

fn report_execution_policy_error(error: &ExecutionPolicyError) -> () {
  eprintln!("error: {error}");

  return;
}

fn report_configuration_error(error: &ConfigError) -> () {
  let message = match *error {
    ConfigError::HomeUnavailable => "the user home directory is unavailable",
    ConfigError::AlreadyExists => "the Kermet configuration already exists",
    ConfigError::Unreadable => "the Kermet configuration cannot be read",
    ConfigError::Unwritable => "the Kermet configuration cannot be written",
    ConfigError::InvalidDocument => "the Kermet configuration is invalid",
    ConfigError::InvalidKey => "the configuration key is not supported",
    ConfigError::InvalidValue => "the configuration value is invalid",
    ConfigError::MissingProvider => "no provider is configured",
    ConfigError::MissingModel => "no model is configured",
    ConfigError::MissingReasoningEffort => "no reasoning effort is configured",
    ConfigError::InvalidSelection(_) => "the model selection is invalid",
  };

  eprintln!("error: {message}");

  return;
}

fn report_provider_registry_error(error: &ProviderRegistryError) -> ExitCode {
  let (exit_code, message) = match *error {
    ProviderRegistryError::UnsupportedProvider => (
      ExitCode::from(CONFIGURATION_EXIT_CODE),
      "the configured model provider is not supported",
    ),
    ProviderRegistryError::MissingCredentials => (
      ExitCode::from(AUTHENTICATION_EXIT_CODE),
      "the selected provider profile is not authenticated",
    ),
    ProviderRegistryError::CredentialStoreUnavailable => (
      ExitCode::from(AUTHENTICATION_EXIT_CODE),
      "secure credential storage is unavailable",
    ),
  };

  eprintln!("error: {message}");

  return exit_code;
}

fn report_agent_loop_error(error: &AgentLoopError) -> () {
  let message = match *error {
    AgentLoopError::Model(ModelProviderError::Exhausted) => "the model provider has no response",
    AgentLoopError::Model(ModelProviderError::MissingCredentials) => {
      "the model provider has no credentials"
    }
    AgentLoopError::Model(ModelProviderError::NotImplemented) => {
      "the configured provider does not implement request execution"
    }
    AgentLoopError::Model(ModelProviderError::RequestFailed) => "the model provider request failed",
    AgentLoopError::Model(ModelProviderError::InvalidResponse) => {
      "the model provider returned an invalid response"
    }
    AgentLoopError::Model(ModelProviderError::UnsupportedToolCall) => {
      "the model provider requested an unsupported tool"
    }
    AgentLoopError::Tool(_) => "a tool execution failed",
    AgentLoopError::TurnLimitExceeded => "the agent reached its turn limit",
  };

  eprintln!("error: {message}");

  return;
}

fn report_thread_store_error(error: &ThreadStoreError) -> () {
  let message = match *error {
    ThreadStoreError::InvalidId => "the thread identifier is invalid",
    ThreadStoreError::NotFound => "the thread does not exist",
    ThreadStoreError::Unreadable => "the thread cannot be read",
    ThreadStoreError::Unwritable => "the thread cannot be saved",
    ThreadStoreError::InvalidThread => "the thread data is invalid",
  };

  eprintln!("error: {message}");

  return;
}

fn render_agent_event(event: AgentEvent, streamed_text: &mut bool) -> () {
  match event {
    AgentEvent::Model(ModelEvent::ThinkingStarted) => println!("Thinking…"),
    AgentEvent::Model(ModelEvent::TextDelta(delta)) => {
      print!("{delta}");
      let _: std::io::Result<()> = std::io::stdout().flush();
      *streamed_text = true;
    }
    AgentEvent::ToolCall { tool_name, target } => {
      println!("Calling {}: {target}", tool_name.function_name())
    }
    AgentEvent::ToolExecution { tool_name, target } => match tool_name {
      crate::tools::ToolName::Bash => println!("Running Bash: {target}…"),
      crate::tools::ToolName::Edit => println!("Editing {target}…"),
      crate::tools::ToolName::Read => println!("Reading {target}…"),
      crate::tools::ToolName::Write => println!("Writing {target}…"),
    },
  }

  return;
}

fn report_turn_error(error: &TurnError) -> ExitCode {
  let exit_code = match *error {
    TurnError::Provider(ref error) => report_provider_registry_error(error),
    TurnError::Agent(ref error) => {
      report_agent_loop_error(error);
      ExitCode::FAILURE
    }
  };

  return exit_code;
}

fn config(arguments: ConfigArgs) -> ExitCode {
  let path = match user_path() {
    Ok(path) => path,
    Err(error) => {
      report_configuration_error(&error);
      return ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  };

  match arguments.command {
    ConfigCommand::Init => match initialize(&path) {
      Ok(()) => println!("Created configuration: {}", path.display()),
      Err(error) => {
        report_configuration_error(&error);
        return ExitCode::from(CONFIGURATION_EXIT_CODE);
      }
    },
    ConfigCommand::Get(arguments) => match get(&path, &arguments.key) {
      Ok(Some(value)) => println!("{value}"),
      Ok(None) => {
        eprintln!("error: the configuration key is not set");
        return ExitCode::from(CONFIGURATION_EXIT_CODE);
      }
      Err(error) => {
        report_configuration_error(&error);
        return ExitCode::from(CONFIGURATION_EXIT_CODE);
      }
    },
    ConfigCommand::Set(arguments) => match set(&path, &arguments.key, arguments.value) {
      Ok(()) => println!("Updated {}", arguments.key),
      Err(error) => {
        report_configuration_error(&error);
        return ExitCode::from(CONFIGURATION_EXIT_CODE);
      }
    },
    ConfigCommand::Unset(arguments) => match unset(&path, &arguments.key) {
      Ok(true) => println!("Removed {}", arguments.key),
      Ok(false) => println!("{} is not set", arguments.key),
      Err(error) => {
        report_configuration_error(&error);
        return ExitCode::from(CONFIGURATION_EXIT_CODE);
      }
    },
    ConfigCommand::Check(arguments) => {
      match resolve_model_selection(&path, &arguments.profile, ModelOverrides::none()) {
        Ok(selection) => {
          println!("Profile: {}", arguments.profile);
          println!("Provider: {}", selection.provider());
          println!("Model: {}", selection.model());
          println!(
            "Reasoning effort: {}",
            selection.reasoning_effort().identifier()
          );
        }
        Err(error) => {
          report_configuration_error(&error);
          return ExitCode::from(CONFIGURATION_EXIT_CODE);
        }
      }
    }
  }

  return ExitCode::SUCCESS;
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

fn hub(arguments: HubArgs) -> ExitCode {
  if arguments.open_picker {
    return hub_open_picker();
  }

  let config_path = match user_path() {
    Ok(path) => path,
    Err(error) => {
      report_configuration_error(&error);
      return ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  };
  let (thread_directory, workspace_directory) = match config_path.parent() {
    Some(parent) => (parent.join("threads"), parent.join("workspaces")),
    None => {
      eprintln!("error: the Kermet data directory cannot be resolved");
      return ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  };

  let exit_code = match serve(thread_directory, workspace_directory, arguments.port) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      report_hub_error(&error);
      ExitCode::FAILURE
    }
  };

  return exit_code;
}

fn hub_open_picker() -> ExitCode {
  let path = FileDialog::new()
    .set_title("Select a workspace folder")
    .pick_folder();
  let path = path.map(|path| {
    let path = path.display().to_string();

    return path;
  });
  let output = serde_json::json!({ "path": path });
  let output = match serde_json::to_string(&output) {
    Ok(output) => output,
    Err(_) => return ExitCode::FAILURE,
  };

  println!("{output}");

  return ExitCode::SUCCESS;
}

fn report_hub_error(error: &HubError) -> () {
  let message = match *error {
    HubError::Runtime => "the hub runtime could not start",
    HubError::Bind => "the hub server could not bind to the requested port",
    HubError::Address => "the hub server address could not be resolved",
    HubError::Serve => "the hub server stopped unexpectedly",
  };

  eprintln!("error: {message}");

  return;
}

fn run_task(arguments: RunArgs) -> ExitCode {
  if arguments.interactive {
    return run_interactive(arguments);
  }
  let execution_policy = match execution_policy(&arguments) {
    Ok(policy) => policy,
    Err(error) => {
      report_execution_policy_error(&error);
      return ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };
  let task = match arguments.task {
    Some(task) => task,
    None => {
      eprintln!("error: a task is required outside interactive mode");

      return ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };
  let config_path = match user_path() {
    Ok(config_path) => config_path,
    Err(error) => {
      report_configuration_error(&error);
      return ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  };
  let overrides = ModelOverrides {
    provider: arguments.provider,
    model: arguments.model,
    reasoning_effort: arguments.reasoning_effort.map(Into::into),
  };
  let model_selection = match resolve_model_selection(&config_path, &arguments.profile, overrides) {
    Ok(model_selection) => model_selection,
    Err(error) => {
      report_configuration_error(&error);
      return ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  };
  let workspace = match open_workspace(arguments.workspace) {
    Ok(workspace) => workspace,
    Err(error) => {
      report_workspace_error(&error);
      return ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };
  let request = match TaskRequest::new(task, workspace, execution_policy) {
    Ok(request) => request,
    Err(error) => {
      report_task_request_error(&error);
      return ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };
  let mut streamed_text = false;
  let mut on_event = |event| render_agent_event(event, &mut streamed_text);
  let message = match execute_turn(
    &request,
    &model_selection,
    &arguments.profile,
    &mut on_event,
  ) {
    Ok(result) => result,
    Err(error) => return report_turn_error(&error),
  };

  if streamed_text {
    println!();
  } else {
    println!("{message}");
  }

  return ExitCode::SUCCESS;
}

fn run_interactive(arguments: RunArgs) -> ExitCode {
  let execution_policy = match execution_policy(&arguments) {
    Ok(policy) => policy,
    Err(error) => {
      report_execution_policy_error(&error);
      return ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };
  let config_path = match user_path() {
    Ok(path) => path,
    Err(error) => {
      report_configuration_error(&error);
      return ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  };
  let directory = match config_path.parent() {
    Some(parent) => parent.join("threads"),
    None => {
      eprintln!("error: the thread directory cannot be resolved");
      return ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  };
  let store = ThreadStore::new(directory);
  let mut thread = match arguments.resume {
    Some(id) => match store.load(&id) {
      Ok(thread) => thread,
      Err(error) => {
        report_thread_store_error(&error);
        return ExitCode::from(CONFIGURATION_EXIT_CODE);
      }
    },
    None => {
      let workspace = match open_workspace(arguments.workspace) {
        Ok(workspace) => workspace,
        Err(error) => {
          report_workspace_error(&error);
          return ExitCode::from(INVALID_INPUT_EXIT_CODE);
        }
      };
      let id = match IdGenerator::generate("thread") {
        Ok(id) => id,
        Err(_) => {
          eprintln!("error: failed to generate a thread identifier");
          return ExitCode::FAILURE;
        }
      };
      let thread = Thread::new(
        id,
        workspace.path().to_path_buf(),
        arguments.profile.clone(),
      );

      match store.save(&thread) {
        Ok(()) => thread,
        Err(error) => {
          report_thread_store_error(&error);
          return ExitCode::from(CONFIGURATION_EXIT_CODE);
        }
      }
    }
  };
  let profile = thread.profile().to_owned();
  let overrides = ModelOverrides {
    provider: arguments.provider,
    model: arguments.model,
    reasoning_effort: arguments.reasoning_effort.map(Into::into),
  };
  let selection = match resolve_model_selection(&config_path, &profile, overrides) {
    Ok(selection) => selection,
    Err(error) => {
      report_configuration_error(&error);
      return ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  };

  println!("Thread: {}", thread.id());
  println!("Type /exit to leave the thread.");
  let transcript = thread.render_transcript();

  if !transcript.is_empty() {
    print!("{transcript}");
  }

  loop {
    print!("> ");
    let _: std::io::Result<()> = io::stdout().flush();
    let mut input = String::new();
    let bytes_read = match io::stdin().read_line(&mut input) {
      Ok(bytes_read) => bytes_read,
      Err(_) => {
        eprintln!("error: failed to read interactive input");
        return ExitCode::FAILURE;
      }
    };
    if bytes_read == 0 {
      break;
    }
    let input = input.trim();
    if input == "/exit" {
      break;
    }
    if input.is_empty() {
      continue;
    }
    let prompt = thread.prompt(input);
    thread.record_user_message(input.to_owned());
    if let Err(error) = store.save(&thread) {
      report_thread_store_error(&error);
      return ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
    let workspace = match open_workspace(thread.workspace().to_path_buf()) {
      Ok(workspace) => workspace,
      Err(error) => {
        report_workspace_error(&error);
        return ExitCode::from(INVALID_INPUT_EXIT_CODE);
      }
    };
    let request = match TaskRequest::new(prompt, workspace, execution_policy) {
      Ok(request) => request,
      Err(error) => {
        report_task_request_error(&error);
        return ExitCode::from(INVALID_INPUT_EXIT_CODE);
      }
    };
    let mut streamed_text = false;
    let mut on_event = |event| render_agent_event(event, &mut streamed_text);
    let message = match execute_turn(&request, &selection, &profile, &mut on_event) {
      Ok(result) => result,
      Err(error) => return report_turn_error(&error),
    };
    if streamed_text {
      println!();
    } else {
      println!("{message}");
    }
    thread.record_assistant_message(message);
    if let Err(error) = store.save(&thread) {
      report_thread_store_error(&error);
      return ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  }

  return ExitCode::SUCCESS;
}

fn execution_policy(arguments: &RunArgs) -> Result<ExecutionPolicy, ExecutionPolicyError> {
  let policy = ExecutionPolicy::new(
    arguments.sandbox.into(),
    arguments.network.into(),
    arguments.approval_policy.into(),
  );

  return policy;
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
    Some(Command::Hub(arguments)) => hub(arguments),
    Some(Command::Run(arguments)) => run_task(arguments),
    Some(Command::Login(arguments)) => login(arguments),
    Some(Command::Logout(arguments)) => logout(arguments),
    Some(Command::Config(arguments)) => config(arguments),
    None => print_help(),
  };

  return exit_code;
}
