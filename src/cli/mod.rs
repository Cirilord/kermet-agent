mod args;
mod workspace;

pub use args::Cli;

use crate::agent::{AgentEvent, AgentLoop, AgentLoopError, TaskRequest, TaskRequestError};
use crate::config::{
  ConfigError, ModelOverrides, get, initialize, resolve_model_selection, set, unset, user_path,
};
use crate::models::{ModelEvent, ModelProviderError};
use crate::providers::codex::auth::{
  AuthorizationRequest, CodexCredentialStore, TokenClient, UreqTokenTransport,
  receive_authorization_code,
};
use crate::providers::{ProviderRegistry, ProviderRegistryError};
use crate::security::{KeyringCredentialStore, PermissionManager};
use crate::tools::ToolRegistry;
use crate::utils::CallbackServer;
use args::{
  Command, ConfigArgs, ConfigCommand, DoctorArgs, LoginArgs, LoginCommand, LogoutArgs, RunArgs,
};
use clap::CommandFactory;
use std::{io::Write, process::ExitCode};
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

fn run_task(arguments: RunArgs) -> ExitCode {
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
  let request = match TaskRequest::new(arguments.task, workspace, arguments.mode.into()) {
    Ok(request) => request,
    Err(error) => {
      report_task_request_error(&error);
      return ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };
  let registry = ProviderRegistry::new(KeyringCredentialStore::new());
  let provider = match registry.resolve(&model_selection, &arguments.profile) {
    Ok(provider) => provider,
    Err(error) => return report_provider_registry_error(&error),
  };
  let tools = ToolRegistry::new(PermissionManager::new());
  let agent_loop = AgentLoop::new(provider, tools);
  let (result, streamed_text) = {
    let mut streamed_text = false;
    let mut on_event = |event: AgentEvent| match event {
      AgentEvent::Model(ModelEvent::ThinkingStarted) => println!("Thinking…"),
      AgentEvent::Model(ModelEvent::TextDelta(delta)) => {
        print!("{delta}");
        let _: std::io::Result<()> = std::io::stdout().flush();
        streamed_text = true;
      }
      AgentEvent::ToolCall { tool_name, target } => {
        println!("Calling {}: {target}", tool_name.function_name());
      }
      AgentEvent::ToolExecution { tool_name, target } => match tool_name {
        crate::tools::ToolName::ReadFile => println!("Reading {target}…"),
      },
    };
    let result = agent_loop.execute(&request, &model_selection, &mut on_event);

    (result, streamed_text)
  };
  let result = match result {
    Ok(result) => result,
    Err(error) => {
      report_agent_loop_error(&error);
      return ExitCode::FAILURE;
    }
  };

  if streamed_text {
    println!();
  } else {
    println!("{}", result.message());
  }

  return ExitCode::SUCCESS;
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
    Some(Command::Config(arguments)) => config(arguments),
    None => print_help(),
  };

  return exit_code;
}
