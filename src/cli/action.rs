use super::commands::{ConfigKeyInput, ConfigSetInput, LoginInput};

pub(crate) enum CliAction {
  GetConfig(ConfigKeyInput),
  SetConfig(ConfigSetInput),
  ShowConfig,
  UnsetConfig(ConfigKeyInput),
  Login(LoginInput),
  ListModels,
  Logout,
}
