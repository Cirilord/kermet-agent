use keyring::{Entry, Error};

use super::{ProviderError, ProviderKind};

const ACTIVE_PROVIDER_ACCOUNT: &str = "active-provider";
const SERVICE: &str = "com.kermet.agent";

pub(super) struct ActiveProviderStore;

impl ActiveProviderStore {
  pub(super) fn load() -> Result<Option<ProviderKind>, ProviderError> {
    let entry = match entry() {
      Ok(entry) => entry,
      Err(error) => return Err(error),
    };
    let identifier = match entry.get_password() {
      Ok(identifier) => identifier,
      Err(Error::NoEntry) => return Ok(None),
      Err(_) => return Err(ProviderError::ActiveProviderState),
    };
    let provider_kind = match ProviderKind::parse(&identifier) {
      Ok(provider_kind) => provider_kind,
      Err(_) => return Err(ProviderError::ActiveProviderState),
    };

    return Ok(Some(provider_kind));
  }

  pub(super) fn save(provider_kind: &ProviderKind) -> Result<(), ProviderError> {
    let entry = match entry() {
      Ok(entry) => entry,
      Err(error) => return Err(error),
    };

    if entry.set_password(provider_kind.identifier()).is_err() {
      return Err(ProviderError::ActiveProviderState);
    }

    return Ok(());
  }

  pub(super) fn clear() -> Result<(), ProviderError> {
    let entry = match entry() {
      Ok(entry) => entry,
      Err(error) => return Err(error),
    };

    match entry.delete_credential() {
      Ok(()) | Err(Error::NoEntry) => return Ok(()),
      Err(_) => return Err(ProviderError::ActiveProviderState),
    }
  }
}

fn entry() -> Result<Entry, ProviderError> {
  let entry = match Entry::new(SERVICE, ACTIVE_PROVIDER_ACCOUNT) {
    Ok(entry) => entry,
    Err(_) => return Err(ProviderError::ActiveProviderState),
  };

  return Ok(entry);
}
