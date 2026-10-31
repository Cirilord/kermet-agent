use crate::{
  models::ReasoningEffort,
  security::{
    ApprovalPolicy,
    sandbox::{NetworkAccess, SandboxMode},
  },
};
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
pub struct HubArgs {
  /// TCP port for the local hub server.
  #[arg(long, default_value_t = 3000)]
  pub port: u16,
  #[arg(long = "open-picker", hide = true)]
  pub open_picker: bool,
}

#[derive(Args, Debug)]
pub struct RunArgs {
  /// Task for Kermet to perform.
  #[arg(
    required_unless_present = "interactive",
    conflicts_with_all = ["interactive", "resume"]
  )]
  pub task: Option<String>,
  /// Keep Kermet open for a sequence of messages.
  #[arg(short, long)]
  pub interactive: bool,
  /// Resume an existing interactive thread.
  #[arg(long, requires = "interactive")]
  pub resume: Option<String>,
  /// Workspace in which the task will run.
  #[arg(long, default_value = ".")]
  pub workspace: PathBuf,
  /// Filesystem boundary for tool execution.
  #[arg(long, value_enum, default_value_t = CliSandboxMode::ReadOnly)]
  pub sandbox: CliSandboxMode,
  /// Network access available to tool execution.
  #[arg(long, value_enum, default_value_t = CliNetworkAccess::Denied)]
  pub network: CliNetworkAccess,
  /// How Kermet handles actions outside the execution policy.
  #[arg(long, value_enum, default_value_t = CliApprovalPolicy::OnRequest)]
  pub approval_policy: CliApprovalPolicy,
  /// Configuration profile that selects the provider and model.
  #[arg(long, default_value = "default")]
  pub profile: String,
  /// Override the configured provider for this task.
  #[arg(long)]
  pub provider: Option<String>,
  /// Override the configured model for this task.
  #[arg(long)]
  pub model: Option<String>,
  /// Override the configured reasoning effort for this task.
  #[arg(long, value_enum)]
  pub reasoning_effort: Option<CliReasoningEffort>,
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

#[derive(Args, Debug)]
pub struct ConfigArgs {
  #[command(subcommand)]
  pub command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
  /// Create an empty user configuration file.
  Init,
  /// Print a configured value.
  Get(ConfigGetArgs),
  /// Set a configured value.
  Set(ConfigSetArgs),
  /// Remove a configured value.
  Unset(ConfigUnsetArgs),
  /// Validate a model profile.
  Check(ConfigCheckArgs),
}

#[derive(Args, Debug)]
pub struct ConfigGetArgs {
  /// A profile key such as profiles.default.model.
  pub key: String,
}

#[derive(Args, Debug)]
pub struct ConfigSetArgs {
  /// A profile key such as profiles.default.model.
  pub key: String,
  /// Value to save for the selected key.
  pub value: String,
}

#[derive(Args, Debug)]
pub struct ConfigUnsetArgs {
  /// A profile key such as profiles.default.model.
  pub key: String,
}

#[derive(Args, Debug)]
pub struct ConfigCheckArgs {
  /// Profile to validate.
  #[arg(long, default_value = "default")]
  pub profile: String,
}

#[derive(Debug, Subcommand)]
pub enum Command {
  /// Inspect the current workspace and local prerequisites.
  Doctor(DoctorArgs),
  /// Start the local thread hub.
  Hub(HubArgs),
  /// Start a task in the selected workspace.
  Run(RunArgs),
  /// Authenticate with Codex through the browser OAuth flow.
  Login(LoginArgs),
  /// Remove saved Codex credentials.
  Logout(LogoutArgs),
  /// Manage non-secret user configuration.
  Config(ConfigArgs),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliSandboxMode {
  ReadOnly,
  WorkspaceWrite,
  DangerFullAccess,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliNetworkAccess {
  Denied,
  Allowed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliApprovalPolicy {
  OnRequest,
  Never,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliReasoningEffort {
  None,
  Minimal,
  Low,
  Medium,
  High,
  #[value(name = "xhigh")]
  XHigh,
  Max,
}

impl From<CliReasoningEffort> for ReasoningEffort {
  fn from(reasoning_effort: CliReasoningEffort) -> Self {
    let value = match reasoning_effort {
      CliReasoningEffort::None => Self::None,
      CliReasoningEffort::Minimal => Self::Minimal,
      CliReasoningEffort::Low => Self::Low,
      CliReasoningEffort::Medium => Self::Medium,
      CliReasoningEffort::High => Self::High,
      CliReasoningEffort::XHigh => Self::XHigh,
      CliReasoningEffort::Max => Self::Max,
    };

    return value;
  }
}

impl From<CliSandboxMode> for SandboxMode {
  fn from(mode: CliSandboxMode) -> Self {
    let sandbox_mode = match mode {
      CliSandboxMode::ReadOnly => Self::ReadOnly,
      CliSandboxMode::WorkspaceWrite => Self::WorkspaceWrite,
      CliSandboxMode::DangerFullAccess => Self::DangerFullAccess,
    };

    return sandbox_mode;
  }
}

impl From<CliNetworkAccess> for NetworkAccess {
  fn from(access: CliNetworkAccess) -> Self {
    let network_access = match access {
      CliNetworkAccess::Denied => Self::Denied,
      CliNetworkAccess::Allowed => Self::Allowed,
    };

    return network_access;
  }
}

impl From<CliApprovalPolicy> for ApprovalPolicy {
  fn from(policy: CliApprovalPolicy) -> Self {
    let approval_policy = match policy {
      CliApprovalPolicy::OnRequest => Self::OnRequest,
      CliApprovalPolicy::Never => Self::Never,
    };

    return approval_policy;
  }
}

#[cfg(test)]
mod tests {
  use super::{
    Cli, CliApprovalPolicy, CliNetworkAccess, CliSandboxMode, Command, ConfigCommand, LoginCommand,
  };
  use clap::Parser;

  #[test]
  fn parses_run_arguments() -> () {
    let cli = Cli::try_parse_from([
      "kermet",
      "run",
      "inspect the authentication flow",
      "--workspace",
      "./workspace",
      "--sandbox",
      "workspace-write",
      "--network",
      "denied",
      "--approval-policy",
      "on-request",
    ])
    .expect("the command-line arguments should parse");

    let Command::Run(arguments) = cli.command.expect("a subcommand should be present") else {
      panic!("the run subcommand should parse");
    };

    assert_eq!(
      arguments.task.as_deref(),
      Some("inspect the authentication flow")
    );
    assert_eq!(arguments.workspace, std::path::PathBuf::from("./workspace"));
    assert_eq!(arguments.sandbox, CliSandboxMode::WorkspaceWrite);
    assert_eq!(arguments.network, CliNetworkAccess::Denied);
    assert_eq!(arguments.approval_policy, CliApprovalPolicy::OnRequest);

    return;
  }

  #[test]
  fn parses_hub_arguments() -> () {
    let cli = Cli::try_parse_from(["kermet", "hub", "--port", "4100"])
      .expect("the command-line arguments should parse");
    let Command::Hub(arguments) = cli.command.expect("a subcommand should be present") else {
      panic!("the hub subcommand should parse");
    };

    assert_eq!(arguments.port, 4100);

    return;
  }

  #[test]
  fn parses_the_hidden_hub_picker_flag() -> () {
    let cli = Cli::try_parse_from(["kermet", "hub", "--open-picker"])
      .expect("the internal hub picker flag should parse");

    let Command::Hub(arguments) = cli.command.expect("a subcommand should be present") else {
      panic!("the hub picker flag should parse");
    };

    assert!(arguments.open_picker);

    return;
  }

  #[test]
  fn parses_an_interactive_run_with_a_thread_to_resume() -> () {
    let cli = Cli::try_parse_from(["kermet", "run", "-i", "--resume", "thread-0123456789abcdef"])
      .expect("the command-line arguments should parse");
    let Command::Run(arguments) = cli.command.expect("a subcommand should be present") else {
      panic!("the run subcommand should parse");
    };

    assert!(arguments.interactive);
    assert_eq!(arguments.resume.as_deref(), Some("thread-0123456789abcdef"));
    assert_eq!(arguments.task, None);

    return;
  }

  #[test]
  fn rejects_a_thread_to_resume_without_interactive_mode() -> () {
    let result = Cli::try_parse_from(["kermet", "run", "--resume", "thread-0123456789abcdef"]);

    assert!(result.is_err());

    return;
  }

  #[test]
  fn rejects_a_task_in_interactive_mode() -> () {
    let result = Cli::try_parse_from(["kermet", "run", "-i", "inspect the workspace"]);

    assert!(result.is_err());

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

  #[test]
  fn parses_config_set() -> () {
    let cli = Cli::try_parse_from([
      "kermet",
      "config",
      "set",
      "profiles.default.model",
      "gpt-6.1-sol",
    ])
    .expect("the command-line arguments should parse");
    let Command::Config(arguments) = cli.command.expect("a subcommand should be present") else {
      panic!("the config subcommand should parse");
    };
    let ConfigCommand::Set(arguments) = arguments.command else {
      panic!("the config set subcommand should parse");
    };

    assert_eq!(arguments.key, "profiles.default.model");
    assert_eq!(arguments.value, "gpt-6.1-sol");

    return;
  }
}
