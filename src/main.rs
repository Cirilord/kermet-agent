mod cli;
mod providers;

const INVALID_INPUT_EXIT_CODE: u8 = 2;
const PROVIDER_EXIT_CODE: u8 = 3;

fn main() -> std::process::ExitCode {
  let action = match cli::parse() {
    Ok(action) => action,
    Err(error) => {
      eprintln!("error: {error}");

      return std::process::ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };

  let exit_code = match action {
    cli::CliAction::Login(args) => login(args),
  };

  return exit_code;
}

fn login(input: cli::LoginInput) -> std::process::ExitCode {
  let provider_kind = match providers::ProviderKind::parse(input.provider()) {
    Ok(provider_kind) => provider_kind,
    Err(error) => {
      eprintln!("error: {error}");

      return std::process::ExitCode::from(PROVIDER_EXIT_CODE);
    }
  };
  let provider = providers::Provider::from(provider_kind);
  let account = match provider.login() {
    Ok(account) => account,
    Err(error) => {
      eprintln!("error: {error}");

      return std::process::ExitCode::from(PROVIDER_EXIT_CODE);
    }
  };

  match account.email() {
    Some(email) => println!("Signed in with {} as {email}.", account.provider_kind()),
    None => println!("Signed in with {}.", account.provider_kind()),
  }

  return std::process::ExitCode::SUCCESS;
}
