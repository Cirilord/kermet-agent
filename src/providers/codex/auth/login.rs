use std::time::{SystemTime, UNIX_EPOCH};

use super::{
  authorization::{create_attempt, is_dynamic_client},
  callback::CallbackServer,
  credentials::{CredentialRecord, CredentialStore},
  error::CodexAuthError,
  oauth::OAuthClient,
};

const REQUIRED_PLAN_SCOPE: &str = "chatgpt.tokens.use.direct";

pub(in super::super) struct CodexAccount {
  email: Option<String>,
}

impl CodexAccount {
  pub(in super::super) fn into_email(self) -> Option<String> {
    return self.email;
  }
}

pub(in super::super) fn login() -> Result<CodexAccount, CodexAuthError> {
  let existing_record = match CredentialStore::load() {
    Ok(record) => record,
    Err(error) => return Err(error),
  };
  let host_id = match CredentialStore::load_or_create_host_id() {
    Ok(host_id) => host_id,
    Err(error) => return Err(error),
  };
  let callback_server = match CallbackServer::start() {
    Ok(callback_server) => callback_server,
    Err(error) => return Err(error),
  };
  let existing_client_id = existing_record.as_ref().map(record_client_id);
  let id_token_hint = existing_record.as_ref().map(record_id_token);
  let attempt = match create_attempt(
    existing_client_id,
    callback_server.redirect_uri().to_owned(),
    &host_id,
    id_token_hint,
  ) {
    Ok(attempt) => attempt,
    Err(error) => return Err(error),
  };

  if webbrowser::open(attempt.authorization_url()).is_err() {
    return Err(CodexAuthError::Browser);
  }
  let callback = match callback_server.wait_for_callback(attempt.state()) {
    Ok(callback) => callback,
    Err(error) => return Err(error),
  };
  let client_id = match resolve_client_id(&attempt.client_id, callback.client_id.as_deref()) {
    Ok(client_id) => client_id,
    Err(error) => return Err(error),
  };
  let oauth = match OAuthClient::new() {
    Ok(oauth) => oauth,
    Err(error) => return Err(error),
  };
  let nonce = attempt.nonce().to_owned();
  let token_set = match oauth.exchange_code(
    &client_id,
    &callback.code,
    attempt.code_verifier,
    &attempt.redirect_uri,
  ) {
    Ok(token_set) => token_set,
    Err(error) => return Err(error),
  };
  let identity = match oauth.validate_id_token(&token_set.id_token, &client_id, &nonce) {
    Ok(identity) => identity,
    Err(error) => return Err(error),
  };

  if let Some(record) = existing_record.as_ref()
    && record.subject != identity.subject
  {
    return Err(CodexAuthError::IdentityValidation);
  }
  let scopes = token_set.scopes;

  let mut has_plan_scope = false;

  for scope in &scopes {
    if scope == REQUIRED_PLAN_SCOPE {
      has_plan_scope = true;

      break;
    }
  }
  if !has_plan_scope {
    return Err(CodexAuthError::MissingPlanPermission);
  }
  let expires_at_unix_seconds = match expiration(token_set.expires_in) {
    Ok(expires_at_unix_seconds) => expires_at_unix_seconds,
    Err(error) => return Err(error),
  };
  let record = CredentialRecord {
    access_token: token_set.access_token,
    client_id,
    email: identity.email.clone(),
    expires_at_unix_seconds,
    id_token: token_set.id_token,
    refresh_token: token_set.refresh_token,
    scopes,
    subject: identity.subject,
  };

  if let Err(error) = CredentialStore::save(&record) {
    return Err(error);
  }

  let account = CodexAccount {
    email: identity.email,
  };

  return Ok(account);
}

pub(super) fn resolve_client_id(
  attempted_client_id: &str,
  callback_client_id: Option<&str>,
) -> Result<String, CodexAuthError> {
  if is_dynamic_client(attempted_client_id) {
    let client_id = match callback_client_id {
      Some(client_id) => client_id,
      None => return Err(CodexAuthError::Callback),
    };

    if client_id.is_empty() || is_dynamic_client(client_id) {
      return Err(CodexAuthError::Callback);
    }

    return Ok(client_id.to_owned());
  }
  if let Some(callback_client_id) = callback_client_id
    && callback_client_id != attempted_client_id
  {
    return Err(CodexAuthError::Callback);
  }

  return Ok(attempted_client_id.to_owned());
}

fn expiration(expires_in: u64) -> Result<u64, CodexAuthError> {
  let duration = match SystemTime::now().duration_since(UNIX_EPOCH) {
    Ok(duration) => duration,
    Err(_) => return Err(CodexAuthError::TokenExchange),
  };
  let now = duration.as_secs();
  let expiration = match now.checked_add(expires_in) {
    Some(expiration) => expiration,
    None => return Err(CodexAuthError::TokenExchange),
  };

  return Ok(expiration);
}

fn record_client_id(record: &CredentialRecord) -> &str {
  return &record.client_id;
}

fn record_id_token(record: &CredentialRecord) -> &str {
  return &record.id_token;
}
