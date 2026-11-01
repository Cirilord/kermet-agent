mod cli;

const INVALID_INPUT_EXIT_CODE: u8 = 2;

fn main() -> std::process::ExitCode {
  let action = match cli::parse() {
    Ok(action) => action,
    Err(error) => {
      eprintln!("error: {error}");

      return std::process::ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };

  match action {
    cli::CliAction::Login(args) => login(args),
  }

  return std::process::ExitCode::SUCCESS;
}

fn login(input: cli::LoginInput) -> () {
  println!("Provider: {}", input.provider());

  return;
}
