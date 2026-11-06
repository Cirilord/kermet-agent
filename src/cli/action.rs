use super::commands::{ConfigKeyInput, ConfigSetInput, DefaultModelInput, LoginInput};

pub(crate) enum CliAction {
  GetConfig(ConfigKeyInput),
  SetConfig(ConfigSetInput),
  ShowConfig,
  UnsetConfig(ConfigKeyInput),
  Login(LoginInput),
  ListModels,
  SetDefaultModel(DefaultModelInput),
  Logout,
}
