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
  let Command::Login(args) = cli.command;
  let action = match commands::login(args) {
    Ok(action) => action,
    Err(_) => panic!("the login action should be valid"),
  };
  let CliAction::Login(args) = action;

  assert_eq!(args.provider(), "example-provider");

  return;
}

#[test]
fn rejects_an_empty_login_provider() -> () {
  let cli =
    Cli::try_parse_from(["kermet", "login", "   "]).expect("the login command should parse");
  let Command::Login(args) = cli.command;
  let result = commands::login(args);

  assert!(matches!(result, Err(CliError::EmptyLoginProvider)));

  return;
}
