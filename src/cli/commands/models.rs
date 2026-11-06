use clap::{Args, Subcommand};

use super::super::{CliAction, CliError};

#[derive(Args)]
pub(in super::super) struct ModelsArgs {
  #[command(subcommand)]
  command: Option<ModelsCommand>,
}

#[derive(Subcommand)]
enum ModelsCommand {
  /// Set the default model profile.
  Default(DefaultModelArgs),
}

#[derive(Args)]
struct DefaultModelArgs {
  slug: String,
}

pub(crate) struct DefaultModelInput {
  slug: String,
}

impl DefaultModelInput {
  pub(crate) fn slug(&self) -> &str {
    return &self.slug;
  }
}

pub(in super::super) fn models(args: ModelsArgs) -> Result<CliAction, CliError> {
  let action = match args.command {
    Some(ModelsCommand::Default(args)) => {
      CliAction::SetDefaultModel(DefaultModelInput { slug: args.slug })
    }
    None => CliAction::ListModels,
  };

  return Ok(action);
}
