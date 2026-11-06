use std::time::{SystemTime, UNIX_EPOCH};

use super::{
  credentials::{CredentialRecord, CredentialStore},
  error::CodexAuthError,
  oauth::OAuthClient,
};

const REFRESH_SKEW_SECONDS: u64 = 120;

pub(in super::super) fn access_token() -> Result<Option<String>, CodexAuthError> {
  let record = match CredentialStore::load() {
    Ok(Some(record)) => record,
    Ok(None) => return Ok(None),
    Err(error) => return Err(error),
  };

  if record.access_token.is_empty() || record.refresh_token.is_empty() {
    return Ok(None);
  }
  if !must_refresh(&record) {
    return Ok(Some(record.access_token));
  }
  let oauth = match OAuthClient::new() {
    Ok(oauth) => oauth,
    Err(error) => return Err(error),
  };
  let token_set = match oauth.refresh_token(&record.client_id, &record.refresh_token) {
    Ok(token_set) => token_set,
    Err(error) => return Err(error),
  };
  let mut record = record;

  record.access_token = token_set.access_token;
  record.expires_at_unix_seconds = match expiration(token_set.expires_in) {
    Ok(expiration) => expiration,
    Err(error) => return Err(error),
  };
  record.refresh_token = token_set.refresh_token;

  if let Some(scopes) = token_set.scopes {
    record.scopes = scopes;
  }

  if let Err(error) = CredentialStore::save(&record) {
    return Err(error);
  }

  return Ok(Some(record.access_token));
}

pub(super) fn must_refresh(record: &CredentialRecord) -> bool {
  let now = match unix_timestamp() {
    Ok(now) => now,
    Err(_) => return true,
  };
  let refresh_at = record
    .expires_at_unix_seconds
    .saturating_sub(REFRESH_SKEW_SECONDS);

  return now >= refresh_at;
}

fn expiration(expires_in: u64) -> Result<u64, CodexAuthError> {
  let now = match unix_timestamp() {
    Ok(now) => now,
    Err(error) => return Err(error),
  };
  let expiration = match now.checked_add(expires_in) {
    Some(expiration) => expiration,
    None => return Err(CodexAuthError::TokenExchange),
  };

  return Ok(expiration);
}

fn unix_timestamp() -> Result<u64, CodexAuthError> {
  let duration = match SystemTime::now().duration_since(UNIX_EPOCH) {
    Ok(duration) => duration,
    Err(_) => return Err(CodexAuthError::TokenExchange),
  };
  let timestamp = duration.as_secs();

  return Ok(timestamp);
}
