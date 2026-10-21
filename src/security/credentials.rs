use keyring::v1::{Entry, Error as KeyringError};
use std::{collections::HashMap, sync::Mutex};

const SERVICE_NAME: &str = "dev.kermet.agent";

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CredentialIdentity {
  provider: String,
  profile: String,
}

impl CredentialIdentity {
  pub fn new(provider: String, profile: String) -> Result<Self, CredentialStoreError> {
    if !is_valid_identity_part(&provider) || !is_valid_identity_part(&profile) {
      return Err(CredentialStoreError::InvalidIdentity);
    }

    return Ok(Self { provider, profile });
  }

  pub fn account_name(&self) -> String {
    let account_name = format!("{}/{}", self.provider, self.profile);

    return account_name;
  }
}

pub struct StoredCredential {
  value: String,
}

impl StoredCredential {
  pub fn new(value: String) -> Result<Self, CredentialStoreError> {
    if value.is_empty() {
      return Err(CredentialStoreError::InvalidCredential);
    }

    return Ok(Self { value });
  }

  pub fn value(&self) -> &str {
    return &self.value;
  }
}

pub trait CredentialStore {
  fn load(
    &self,
    identity: &CredentialIdentity,
  ) -> Result<Option<StoredCredential>, CredentialStoreError>;

  fn save(
    &self,
    identity: &CredentialIdentity,
    credential: &StoredCredential,
  ) -> Result<(), CredentialStoreError>;

  fn delete(&self, identity: &CredentialIdentity) -> Result<(), CredentialStoreError>;
}

pub struct KeyringCredentialStore;

impl KeyringCredentialStore {
  pub fn new() -> Self {
    return Self;
  }

  fn entry(identity: &CredentialIdentity) -> Result<Entry, CredentialStoreError> {
    let account_name = identity.account_name();
    let entry = match Entry::new(SERVICE_NAME, &account_name) {
      Ok(entry) => entry,
      Err(_) => return Err(CredentialStoreError::Unavailable),
    };

    return Ok(entry);
  }
}

impl Default for KeyringCredentialStore {
  fn default() -> Self {
    return Self::new();
  }
}

impl CredentialStore for KeyringCredentialStore {
  fn load(
    &self,
    identity: &CredentialIdentity,
  ) -> Result<Option<StoredCredential>, CredentialStoreError> {
    let entry = match Self::entry(identity) {
      Ok(entry) => entry,
      Err(error) => return Err(error),
    };
    let value = match entry.get_password() {
      Ok(value) => value,
      Err(KeyringError::NoEntry) => return Ok(None),
      Err(_) => return Err(CredentialStoreError::Unavailable),
    };
    let credential = match StoredCredential::new(value) {
      Ok(credential) => credential,
      Err(error) => return Err(error),
    };

    return Ok(Some(credential));
  }

  fn save(
    &self,
    identity: &CredentialIdentity,
    credential: &StoredCredential,
  ) -> Result<(), CredentialStoreError> {
    let entry = match Self::entry(identity) {
      Ok(entry) => entry,
      Err(error) => return Err(error),
    };

    if entry.set_password(credential.value()).is_err() {
      return Err(CredentialStoreError::Unavailable);
    }

    return Ok(());
  }

  fn delete(&self, identity: &CredentialIdentity) -> Result<(), CredentialStoreError> {
    let entry = match Self::entry(identity) {
      Ok(entry) => entry,
      Err(error) => return Err(error),
    };

    match entry.delete_credential() {
      Ok(()) | Err(KeyringError::NoEntry) => return Ok(()),
      Err(_) => return Err(CredentialStoreError::Unavailable),
    }
  }
}

pub struct EphemeralCredentialStore {
  credentials: Mutex<HashMap<CredentialIdentity, StoredCredential>>,
}

impl EphemeralCredentialStore {
  pub fn new() -> Self {
    return Self {
      credentials: Mutex::new(HashMap::new()),
    };
  }
}

impl Default for EphemeralCredentialStore {
  fn default() -> Self {
    return Self::new();
  }
}

impl CredentialStore for EphemeralCredentialStore {
  fn load(
    &self,
    identity: &CredentialIdentity,
  ) -> Result<Option<StoredCredential>, CredentialStoreError> {
    let credentials = match self.credentials.lock() {
      Ok(credentials) => credentials,
      Err(_) => return Err(CredentialStoreError::Unavailable),
    };
    let credential = match credentials.get(identity) {
      Some(credential) => StoredCredential::new(credential.value.clone()),
      None => return Ok(None),
    };

    let credential = match credential {
      Ok(credential) => credential,
      Err(error) => return Err(error),
    };

    return Ok(Some(credential));
  }

  fn save(
    &self,
    identity: &CredentialIdentity,
    credential: &StoredCredential,
  ) -> Result<(), CredentialStoreError> {
    let mut credentials = match self.credentials.lock() {
      Ok(credentials) => credentials,
      Err(_) => return Err(CredentialStoreError::Unavailable),
    };
    let value = match StoredCredential::new(credential.value.clone()) {
      Ok(value) => value,
      Err(error) => return Err(error),
    };

    credentials.insert(identity.clone(), value);

    return Ok(());
  }

  fn delete(&self, identity: &CredentialIdentity) -> Result<(), CredentialStoreError> {
    let mut credentials = match self.credentials.lock() {
      Ok(credentials) => credentials,
      Err(_) => return Err(CredentialStoreError::Unavailable),
    };

    credentials.remove(identity);

    return Ok(());
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum CredentialStoreError {
  InvalidIdentity,
  InvalidCredential,
  Unavailable,
}

fn is_valid_identity_part(value: &str) -> bool {
  if value.is_empty() {
    return false;
  }

  return value
    .bytes()
    .all(|byte| return byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
}

#[cfg(test)]
mod tests {
  use super::{
    CredentialIdentity, CredentialStore, CredentialStoreError, EphemeralCredentialStore,
    StoredCredential,
  };

  #[test]
  fn ephemeral_store_round_trips_a_credential() -> () {
    let store = EphemeralCredentialStore::new();
    let identity = CredentialIdentity::new("codex".to_owned(), "default".to_owned())
      .expect("the identity should be valid");
    let credential =
      StoredCredential::new("secret".to_owned()).expect("the credential should be valid");

    store
      .save(&identity, &credential)
      .expect("the credential should be stored");
    let loaded = store
      .load(&identity)
      .expect("the credential should be loaded")
      .expect("the credential should exist");

    assert_eq!(loaded.value(), "secret");

    return;
  }

  #[test]
  fn deleting_an_ephemeral_credential_is_idempotent() -> () {
    let store = EphemeralCredentialStore::new();
    let identity = CredentialIdentity::new("codex".to_owned(), "default".to_owned())
      .expect("the identity should be valid");

    store
      .delete(&identity)
      .expect("deleting a missing credential should succeed");

    return;
  }

  #[test]
  fn rejects_an_invalid_identity() -> () {
    let error = CredentialIdentity::new("codex".to_owned(), "default/profile".to_owned())
      .expect_err("the identity should be rejected");

    assert_eq!(error, CredentialStoreError::InvalidIdentity);

    return;
  }
}
