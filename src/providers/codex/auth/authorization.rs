use oauth2::{CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope};

use super::{error::CodexAuthError, oauth::create_client};

const DYNAMIC_CLIENT_ID: &str = "dynamic_agent_client";
const RESOURCE: &str = "https://api.openai.com/v1";

pub(super) struct AuthorizationAttempt {
  authorization_url: String,
  pub(super) client_id: String,
  pub(super) code_verifier: PkceCodeVerifier,
  nonce: CsrfToken,
  pub(super) redirect_uri: String,
  state: CsrfToken,
}

impl AuthorizationAttempt {
  pub(super) fn authorization_url(&self) -> &str {
    return &self.authorization_url;
  }

  pub(super) fn nonce(&self) -> &str {
    return self.nonce.secret();
  }

  pub(super) fn state(&self) -> &CsrfToken {
    return &self.state;
  }
}

pub(super) fn create_attempt(
  client_id: Option<&str>,
  redirect_uri: String,
  host_id: &str,
  id_token_hint: Option<&str>,
) -> Result<AuthorizationAttempt, CodexAuthError> {
  let client_id = client_id.unwrap_or(DYNAMIC_CLIENT_ID).to_owned();
  let client = match create_client(&client_id, &redirect_uri) {
    Ok(client) => client,
    Err(error) => return Err(error),
  };
  let nonce = CsrfToken::new_random();
  let (code_challenge, code_verifier) = PkceCodeChallenge::new_random_sha256();
  let request = client
    .authorize_url(CsrfToken::new_random)
    .add_scope(Scope::new("openid".to_owned()))
    .add_scope(Scope::new("profile".to_owned()))
    .add_scope(Scope::new("email".to_owned()))
    .add_scope(Scope::new("offline_access".to_owned()))
    .add_scope(Scope::new("resource.invoke".to_owned()))
    .add_scope(Scope::new("chatgpt.tokens.use.direct".to_owned()))
    .set_pkce_challenge(code_challenge)
    .add_extra_param("ext_agent_host_id", host_id)
    .add_extra_param("resource", RESOURCE)
    .add_extra_param("nonce", nonce.secret());
  let request = if is_dynamic_client(&client_id) {
    request.add_extra_param("agent_name_hint", "Kermet")
  } else {
    request
  };
  let request = match id_token_hint {
    Some(id_token_hint) => request.add_extra_param("id_token_hint", id_token_hint),
    None => request,
  };
  let (authorization_url, state) = request.url();
  let attempt = AuthorizationAttempt {
    authorization_url: authorization_url.into(),
    client_id,
    code_verifier,
    nonce,
    redirect_uri,
    state,
  };

  return Ok(attempt);
}

pub(super) fn is_dynamic_client(client_id: &str) -> bool {
  return client_id == DYNAMIC_CLIENT_ID;
}
