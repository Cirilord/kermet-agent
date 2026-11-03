use super::{credentials::CredentialStore, error::CodexAuthError, oauth::OAuthClient};

pub(in super::super) enum CodexLogout {
  NotSignedIn,
  RevocationUnconfirmed,
  Revoked,
}

pub(in super::super) fn logout() -> Result<CodexLogout, CodexAuthError> {
  let record = match CredentialStore::load() {
    Ok(Some(record)) => record,
    Ok(None) => return Ok(CodexLogout::NotSignedIn),
    Err(error) => return Err(error),
  };

  let result = if record.refresh_token.is_empty() {
    CodexLogout::NotSignedIn
  } else {
    let revocation_confirmed = match OAuthClient::new() {
      Ok(oauth) => oauth.revoke_refresh_token(&record.client_id, &record.refresh_token),
      Err(_) => false,
    };

    if revocation_confirmed {
      CodexLogout::Revoked
    } else {
      CodexLogout::RevocationUnconfirmed
    }
  };

  if let Err(error) = CredentialStore::clear_tokens(record) {
    return Err(error);
  }

  return Ok(result);
}
