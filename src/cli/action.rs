use super::commands::LoginInput;

pub(crate) enum CliAction {
  ShowConfig,
  Login(LoginInput),
  ListModels,
  Logout,
}
