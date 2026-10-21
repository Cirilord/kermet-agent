use clap::Parser;
use kermet::cli::Cli;
use std::process::ExitCode;

fn main() -> ExitCode {
  let cli = Cli::parse();

  return kermet::cli::execute(cli);
}
