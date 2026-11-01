use keyring::{Entry, Error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::CodexAuthError;

const SERVICE: &str = "com.kermet.agent";
const CODEX_CREDENTIAL_ACCOUNT: &str = "codex-credentials";
const CODEX_HOST_ACCOUNT: &str = "codex-host";

#[derive(Deserialize, Serialize)]
pub(super) struct CredentialRecord {
  pub(super) access_token: String,
  pub(super) client_id: String,
  pub(super) email: Option<String>,
  pub(super) expires_at_unix_seconds: u64,
  pub(super) id_token: String,
  pub(super) refresh_token: String,
  pub(super) scopes: Vec<String>,
  pub(super) subject: String,
}

pub(super) struct CredentialStore;

impl CredentialStore {
  pub(super) fn load() -> Result<Option<CredentialRecord>, CodexAuthError> {
    let entry = match credential_entry(CODEX_CREDENTIAL_ACCOUNT) {
      Ok(entry) => entry,
      Err(error) => return Err(error),
    };
    let password = match entry.get_password() {
      Ok(password) => password,
      Err(Error::NoEntry) => return Ok(None),
      Err(_) => return Err(CodexAuthError::Credentials),
    };
    let record = match serde_json::from_str(&password) {
      Ok(record) => record,
      Err(_) => return Err(CodexAuthError::Credentials),
    };

    return Ok(Some(record));
  }

  pub(super) fn load_or_create_host_id() -> Result<String, CodexAuthError> {
    let entry = match credential_entry(CODEX_HOST_ACCOUNT) {
      Ok(entry) => entry,
      Err(error) => return Err(error),
    };
    let host_id = match entry.get_password() {
      Ok(host_id) => host_id,
      Err(Error::NoEntry) => {
        let host_id = format!("urn:uuid:{}", Uuid::new_v4());
        if entry.set_password(&host_id).is_err() {
          return Err(CodexAuthError::Credentials);
        }

        return Ok(host_id);
      }
      Err(_) => return Err(CodexAuthError::Credentials),
    };

    return Ok(host_id);
  }

  pub(super) fn save(record: &CredentialRecord) -> Result<(), CodexAuthError> {
    let serialized = match serde_json::to_string(record) {
      Ok(serialized) => serialized,
      Err(_) => return Err(CodexAuthError::Credentials),
    };
    let entry = match credential_entry(CODEX_CREDENTIAL_ACCOUNT) {
      Ok(entry) => entry,
      Err(error) => return Err(error),
    };

    if entry.set_password(&serialized).is_err() {
      return Err(CodexAuthError::Credentials);
    }

    return Ok(());
  }
}

fn credential_entry(account: &str) -> Result<Entry, CodexAuthError> {
  let entry = match Entry::new(SERVICE, account) {
    Ok(entry) => entry,
    Err(_) => return Err(CodexAuthError::Credentials),
  };

  return Ok(entry);
}
