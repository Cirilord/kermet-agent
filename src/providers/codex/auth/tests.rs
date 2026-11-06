use oauth2::CsrfToken;
use url::Url;

use super::{
  authorization::create_attempt, callback::callback_from_target, credentials::CredentialRecord,
  login::resolve_client_id, session::must_refresh,
};

#[test]
fn includes_the_initial_dynamic_registration_parameters() -> () {
  let attempt = match create_attempt(
    None,
    "http://127.0.0.1:1455/auth/callback".to_owned(),
    "urn:uuid:test-host",
    None,
  ) {
    Ok(attempt) => attempt,
    Err(_) => panic!("the authorization attempt should be valid"),
  };
  let url = Url::parse(attempt.authorization_url()).expect("the authorization URL should parse");
  let parameters = url.query_pairs().collect::<Vec<_>>();

  assert_eq!(
    parameter(&parameters, "client_id"),
    Some("dynamic_agent_client")
  );
  assert_eq!(parameter(&parameters, "agent_name_hint"), Some("Kermet"));
  assert_eq!(
    parameter(&parameters, "ext_agent_host_id"),
    Some("urn:uuid:test-host")
  );
  assert_eq!(
    parameter(&parameters, "state"),
    Some(attempt.state().secret().as_str())
  );
  assert_eq!(
    parameter(&parameters, "code_challenge_method"),
    Some("S256")
  );

  return;
}

#[test]
fn resolves_an_issued_client_id_after_initial_registration() -> () {
  let client_id = match resolve_client_id("dynamic_agent_client", Some("oaiapp_kermet")) {
    Ok(client_id) => client_id,
    Err(_) => panic!("the issued client ID should be accepted"),
  };

  assert_eq!(client_id, "oaiapp_kermet");

  return;
}

#[test]
fn rejects_a_replaced_client_id_during_reauthorization() -> () {
  let result = resolve_client_id("oaiapp_kermet", Some("oaiapp_other"));

  assert!(result.is_err());

  return;
}

#[test]
fn rejects_a_callback_with_an_unexpected_state() -> () {
  let result = callback_from_target(
    "/auth/callback?code=authorization-code&state=unexpected-state&client_id=oaiapp_kermet",
    &CsrfToken::new("expected-state".to_owned()),
  );

  assert!(result.is_err());

  return;
}

#[test]
fn refreshes_a_token_that_is_expired() -> () {
  let record = credential_record(0);

  assert!(must_refresh(&record));

  return;
}

#[test]
fn keeps_a_token_that_is_not_near_expiration() -> () {
  let record = credential_record(u64::MAX);

  assert!(!must_refresh(&record));

  return;
}

fn credential_record(expires_at_unix_seconds: u64) -> CredentialRecord {
  let record = CredentialRecord {
    access_token: "access-token".to_owned(),
    client_id: "client-id".to_owned(),
    email: None,
    expires_at_unix_seconds,
    id_token: "id-token".to_owned(),
    refresh_token: "refresh-token".to_owned(),
    scopes: Vec::new(),
    subject: "subject".to_owned(),
  };

  return record;
}

fn parameter<'a>(
  parameters: &'a [(std::borrow::Cow<'a, str>, std::borrow::Cow<'a, str>)],
  name: &str,
) -> Option<&'a str> {
  let value = parameters
    .iter()
    .find(|(parameter_name, _)| parameter_name == name)
    .map(|(_, value)| value.as_ref());

  return value;
}
