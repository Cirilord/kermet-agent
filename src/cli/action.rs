use super::commands::LoginInput;

pub(crate) enum CliAction {
  Login(LoginInput),
  Logout,
}
