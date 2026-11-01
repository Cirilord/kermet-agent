use std::fmt::{Display, Formatter};

pub(crate) enum CodexAuthError {
  AuthorizationDenied,
  AuthorizationRequest,
  Browser,
  Callback,
  CallbackTimeout,
  Credentials,
  IdentityValidation,
  MissingPlanPermission,
  TokenExchange,
}

impl Display for CodexAuthError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = match *self {
      Self::AuthorizationDenied => formatter.write_str("Codex sign-in was denied"),
      Self::AuthorizationRequest => {
        formatter.write_str("could not create the Codex authorization request")
      }
      Self::Browser => formatter.write_str("could not open the browser for Codex sign-in"),
      Self::Callback => formatter.write_str("received an invalid Codex authorization callback"),
      Self::CallbackTimeout => formatter.write_str("timed out waiting for Codex sign-in"),
      Self::Credentials => formatter.write_str("could not access secure Codex credentials"),
      Self::IdentityValidation => {
        formatter.write_str("could not validate the Codex identity token")
      }
      Self::MissingPlanPermission => {
        formatter.write_str("ChatGPT plan usage was not granted for Codex")
      }
      Self::TokenExchange => formatter.write_str("could not exchange the Codex authorization code"),
    };

    return result;
  }
}
