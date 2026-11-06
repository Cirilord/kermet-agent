use super::{
  CliAction, CliError,
  command::{Cli, Command},
  commands,
};
use clap::Parser;

#[test]
fn parses_the_login_command() -> () {
  let cli = Cli::try_parse_from(["kermet", "login", "example-provider"])
    .expect("the login command should parse");

  assert!(matches!(cli.command, Command::Login(_)));

  return;
}

#[test]
fn trims_the_login_provider() -> () {
  let cli = Cli::try_parse_from(["kermet", "login", "  example-provider  "])
    .expect("the login command should parse");
  let args = match cli.command {
    Command::Config(_) => panic!("the command should be login"),
    Command::Login(args) => args,
    Command::Models => panic!("the command should be login"),
    Command::Logout => panic!("the command should be login"),
  };
  let action = match commands::login(args) {
    Ok(action) => action,
    Err(_) => panic!("the login action should be valid"),
  };
  let args = match action {
    CliAction::ShowConfig => panic!("the action should be login"),
    CliAction::Login(args) => args,
    CliAction::ListModels => panic!("the action should be login"),
    CliAction::Logout => panic!("the action should be login"),
  };

  assert_eq!(args.provider(), "example-provider");

  return;
}

#[test]
fn rejects_an_empty_login_provider() -> () {
  let cli =
    Cli::try_parse_from(["kermet", "login", "   "]).expect("the login command should parse");
  let args = match cli.command {
    Command::Config(_) => panic!("the command should be login"),
    Command::Login(args) => args,
    Command::Models => panic!("the command should be login"),
    Command::Logout => panic!("the command should be login"),
  };
  let result = commands::login(args);

  assert!(matches!(result, Err(CliError::EmptyProvider)));

  return;
}

#[test]
fn parses_the_logout_command() -> () {
  let cli = Cli::try_parse_from(["kermet", "logout"]).expect("the logout command should parse");

  assert!(matches!(cli.command, Command::Logout));

  return;
}

#[test]
fn parses_the_models_command() -> () {
  let cli = Cli::try_parse_from(["kermet", "models"]).expect("the models command should parse");

  assert!(matches!(cli.command, Command::Models));

  return;
}

#[test]
fn parses_the_config_show_command() -> () {
  let cli = Cli::try_parse_from(["kermet", "config", "show"])
    .expect("the config show command should parse");

  assert!(matches!(cli.command, Command::Config(_)));

  return;
}
