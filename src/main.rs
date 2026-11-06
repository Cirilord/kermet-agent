mod cli;
mod config;
mod providers;

const INVALID_INPUT_EXIT_CODE: u8 = 2;
const CONFIGURATION_EXIT_CODE: u8 = 4;
const PROVIDER_EXIT_CODE: u8 = 3;

fn main() -> std::process::ExitCode {
  let action = match cli::parse() {
    Ok(action) => action,
    Err(error) => {
      eprintln!("error: {error}");

      return std::process::ExitCode::from(INVALID_INPUT_EXIT_CODE);
    }
  };

  let initialization = config::ensure_exists();
  if let Err(error) = initialization {
    eprintln!("error: {error}");

    return std::process::ExitCode::from(CONFIGURATION_EXIT_CODE);
  }

  let exit_code = match action {
    cli::CliAction::GetConfig(input) => get_config(input),
    cli::CliAction::SetConfig(input) => set_config(input),
    cli::CliAction::ShowConfig => show_config(),
    cli::CliAction::UnsetConfig(input) => unset_config(input),
    cli::CliAction::Login(args) => login(args),
    cli::CliAction::ListModels => list_models(),
    cli::CliAction::Logout => logout(),
  };

  return exit_code;
}

fn get_config(input: cli::ConfigKeyInput) -> std::process::ExitCode {
  let value = match config::get(input.key()) {
    Ok(value) => value,
    Err(error) => {
      eprintln!("error: {error}");

      return std::process::ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  };

  println!("{value}");

  return std::process::ExitCode::SUCCESS;
}

fn set_config(input: cli::ConfigSetInput) -> std::process::ExitCode {
  let result = config::set(input.key(), input.value());

  if let Err(error) = result {
    eprintln!("error: {error}");

    return std::process::ExitCode::from(CONFIGURATION_EXIT_CODE);
  }

  return std::process::ExitCode::SUCCESS;
}

fn show_config() -> std::process::ExitCode {
  let contents = match config::read() {
    Ok(contents) => contents,
    Err(error) => {
      eprintln!("error: {error}");

      return std::process::ExitCode::from(CONFIGURATION_EXIT_CODE);
    }
  };

  print!("{contents}");

  return std::process::ExitCode::SUCCESS;
}

fn unset_config(input: cli::ConfigKeyInput) -> std::process::ExitCode {
  let result = config::unset(input.key());

  if let Err(error) = result {
    eprintln!("error: {error}");

    return std::process::ExitCode::from(CONFIGURATION_EXIT_CODE);
  }

  return std::process::ExitCode::SUCCESS;
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

fn logout() -> std::process::ExitCode {
  let result = match providers::Provider::logout() {
    Ok(result) => result,
    Err(error) => {
      eprintln!("error: {error}");

      return std::process::ExitCode::from(PROVIDER_EXIT_CODE);
    }
  };

  match result {
    providers::ProviderLogout::NotSignedIn => println!("No active provider session was found."),
    providers::ProviderLogout::Revoked => println!("Signed out."),
    providers::ProviderLogout::RevocationUnconfirmed => {
      eprintln!("Signed out locally. Remote session revocation was not confirmed.");
    }
  }

  return std::process::ExitCode::SUCCESS;
}

fn list_models() -> std::process::ExitCode {
  let models = match providers::Provider::list_models() {
    Ok(models) => models,
    Err(error) => {
      eprintln!("error: {error}");

      return std::process::ExitCode::from(PROVIDER_EXIT_CODE);
    }
  };

  for model in models {
    println!("{}\t{}", model.slug(), model.display_name());
  }

  return std::process::ExitCode::SUCCESS;
}
