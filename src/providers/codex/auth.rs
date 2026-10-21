use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime};

use crate::security::{
  CredentialIdentity, CredentialStore, CredentialStoreError, StoredCredential,
};
use crate::utils::{CallbackServer, CallbackServerError};

const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const SCOPE: &str = "openid profile email offline_access";
const REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const REFRESH_SKEW: Duration = Duration::from_secs(120);
const TOKEN_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const TOKEN_RESPONSE_SIZE_LIMIT: u64 = 16 * 1024;

pub struct CodexOAuthConfig;

impl CodexOAuthConfig {
  pub fn client_id() -> &'static str {
    return CLIENT_ID;
  }

  pub fn authorize_url() -> &'static str {
    return AUTHORIZE_URL;
  }

  pub fn token_url() -> &'static str {
    return TOKEN_URL;
  }

  pub fn scope() -> &'static str {
    return SCOPE;
  }
}

pub struct PkceCodes {
  verifier: String,
  challenge: String,
}

impl PkceCodes {
  pub fn generate() -> Self {
    let mut random_bytes = [0_u8; 32];
    rand::rng().fill(&mut random_bytes);
    let verifier = URL_SAFE_NO_PAD.encode(random_bytes);
    let digest = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(digest);

    return Self {
      verifier,
      challenge,
    };
  }

  pub fn challenge(&self) -> &str {
    return &self.challenge;
  }

  pub fn verifier(&self) -> &str {
    return &self.verifier;
  }
}

pub struct AuthorizationRequest {
  pkce: PkceCodes,
  state: String,
}

impl AuthorizationRequest {
  pub fn new() -> Self {
    let mut random_bytes = [0_u8; 32];
    rand::rng().fill(&mut random_bytes);
    let state = URL_SAFE_NO_PAD.encode(random_bytes);

    return Self {
      pkce: PkceCodes::generate(),
      state,
    };
  }

  pub fn authorization_url(&self) -> String {
    let redirect_uri = REDIRECT_URI.replace(":", "%3A").replace("/", "%2F");
    let url = format!(
      "{AUTHORIZE_URL}?response_type=code&client_id={CLIENT_ID}&redirect_uri={redirect_uri}&scope=openid%20profile%20email%20offline_access&code_challenge={}&code_challenge_method=S256&state={}&id_token_add_organizations=true&codex_cli_simplified_flow=true",
      self.pkce.challenge(),
      self.state
    );

    return url;
  }

  pub fn state(&self) -> &str {
    return &self.state;
  }
}

impl Default for AuthorizationRequest {
  fn default() -> Self {
    return Self::new();
  }
}

pub fn validate_callback(callback: &str, expected_state: &str) -> Result<String, CallbackError> {
  let query = match callback.split_once('?') {
    Some((_, query)) => query,
    None => return Err(CallbackError::MissingCode),
  };
  let mut code = None;
  let mut state = None;

  for parameter in query.split('&') {
    let (name, value) = match parameter.split_once('=') {
      Some(parameter) => parameter,
      None => continue,
    };

    match name {
      "code" => code = Some(value),
      "state" => state = Some(value),
      _ => {}
    }
  }

  if state != Some(expected_state) {
    return Err(CallbackError::InvalidState);
  }

  let code = match code {
    Some(code) if !code.is_empty() => code.to_owned(),
    _ => return Err(CallbackError::MissingCode),
  };

  return Ok(code);
}

pub fn receive_authorization_code(
  server: &CallbackServer,
  request: &AuthorizationRequest,
) -> Result<String, AuthorizationError> {
  let callback = match server.receive() {
    Ok(callback) => callback,
    Err(error) => return Err(AuthorizationError::CallbackServer(error)),
  };
  let code = match validate_callback(&callback, request.state()) {
    Ok(code) => code,
    Err(error) => return Err(AuthorizationError::Callback(error)),
  };

  return Ok(code);
}

#[derive(Debug)]
pub enum AuthorizationError {
  CallbackServer(CallbackServerError),
  Callback(CallbackError),
}

pub trait TokenTransport {
  fn post_form(&self, url: &str, fields: &[(&str, &str)]) -> Result<String, TokenTransportError>;
}

pub struct UreqTokenTransport {
  agent: ureq::Agent,
}

impl UreqTokenTransport {
  pub fn new() -> Self {
    let config = ureq::Agent::config_builder()
      .https_only(true)
      .timeout_global(Some(TOKEN_REQUEST_TIMEOUT))
      .build();

    return Self {
      agent: config.new_agent(),
    };
  }
}

impl Default for UreqTokenTransport {
  fn default() -> Self {
    return Self::new();
  }
}

impl TokenTransport for UreqTokenTransport {
  fn post_form(&self, url: &str, fields: &[(&str, &str)]) -> Result<String, TokenTransportError> {
    let mut response = match self.agent.post(url).send_form(fields.iter().copied()) {
      Ok(response) => response,
      Err(_) => return Err(TokenTransportError::RequestFailed),
    };
    let body = match response
      .body_mut()
      .with_config()
      .limit(TOKEN_RESPONSE_SIZE_LIMIT)
      .read_to_string()
    {
      Ok(body) => body,
      Err(_) => return Err(TokenTransportError::ResponseReadFailed),
    };

    return Ok(body);
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenTransportError {
  RequestFailed,
  ResponseReadFailed,
}

pub struct TokenClient<T> {
  transport: T,
}

impl<T> TokenClient<T>
where
  T: TokenTransport,
{
  pub fn new(transport: T) -> Self {
    return Self { transport };
  }

  pub fn exchange_authorization_code(
    &self,
    code: &str,
    request: &AuthorizationRequest,
    now: SystemTime,
  ) -> Result<CodexTokens, TokenExchangeError> {
    if code.is_empty() {
      return Err(TokenExchangeError::MissingAuthorizationCode);
    }

    let fields = [
      ("grant_type", "authorization_code"),
      ("client_id", CLIENT_ID),
      ("code", code),
      ("redirect_uri", REDIRECT_URI),
      ("code_verifier", request.pkce.verifier()),
    ];

    let tokens = self.exchange(&fields, now);

    return tokens;
  }

  pub fn refresh(
    &self,
    refresh_token: &str,
    now: SystemTime,
  ) -> Result<CodexTokens, TokenExchangeError> {
    if refresh_token.is_empty() {
      return Err(TokenExchangeError::MissingRefreshToken);
    }

    let fields = [
      ("grant_type", "refresh_token"),
      ("client_id", CLIENT_ID),
      ("refresh_token", refresh_token),
    ];

    let tokens = self.exchange(&fields, now);

    return tokens;
  }

  fn exchange(
    &self,
    fields: &[(&str, &str)],
    now: SystemTime,
  ) -> Result<CodexTokens, TokenExchangeError> {
    let body = match self.transport.post_form(TOKEN_URL, fields) {
      Ok(body) => body,
      Err(error) => return Err(TokenExchangeError::Transport(error)),
    };
    let response = match serde_json::from_str::<TokenResponse>(&body) {
      Ok(response) => response,
      Err(_) => return Err(TokenExchangeError::InvalidResponse),
    };
    if response.access_token.is_empty()
      || response.refresh_token.is_empty()
      || response.expires_in == 0
    {
      return Err(TokenExchangeError::InvalidResponse);
    }
    let expires_at = match now.checked_add(Duration::from_secs(response.expires_in)) {
      Some(expires_at) => expires_at,
      None => return Err(TokenExchangeError::InvalidExpiration),
    };

    return Ok(CodexTokens::new(
      response.access_token,
      response.refresh_token,
      expires_at,
    ));
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TokenExchangeError {
  MissingAuthorizationCode,
  MissingRefreshToken,
  Transport(TokenTransportError),
  InvalidResponse,
  InvalidExpiration,
}

#[derive(Deserialize)]
struct TokenResponse {
  access_token: String,
  refresh_token: String,
  expires_in: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CallbackError {
  InvalidState,
  MissingCode,
}

pub struct CodexTokens {
  access_token: String,
  refresh_token: String,
  expires_at: SystemTime,
}

impl CodexTokens {
  pub fn new(access_token: String, refresh_token: String, expires_at: SystemTime) -> Self {
    return Self {
      access_token,
      refresh_token,
      expires_at,
    };
  }

  pub fn should_refresh(&self, now: SystemTime) -> bool {
    let refresh_at = match self.expires_at.checked_sub(REFRESH_SKEW) {
      Some(refresh_at) => refresh_at,
      None => return true,
    };

    return now >= refresh_at;
  }

  pub fn access_token(&self) -> &str {
    return &self.access_token;
  }

  pub fn refresh_token(&self) -> &str {
    return &self.refresh_token;
  }

  pub fn expires_at(&self) -> SystemTime {
    return self.expires_at;
  }
}

pub struct CodexCredentialStore<S> {
  store: S,
  identity: CredentialIdentity,
}

impl<S> CodexCredentialStore<S>
where
  S: CredentialStore,
{
  pub fn new(store: S, profile: String) -> Result<Self, CodexCredentialStoreError> {
    let identity = match CredentialIdentity::new("codex".to_owned(), profile) {
      Ok(identity) => identity,
      Err(error) => return Err(CodexCredentialStoreError::CredentialStore(error)),
    };

    return Ok(Self { store, identity });
  }

  pub fn load(&self) -> Result<Option<CodexTokens>, CodexCredentialStoreError> {
    let credential = match self.store.load(&self.identity) {
      Ok(credential) => credential,
      Err(error) => return Err(CodexCredentialStoreError::CredentialStore(error)),
    };
    let credential = match credential {
      Some(credential) => credential,
      None => return Ok(None),
    };
    let stored_tokens = match serde_json::from_str::<StoredCodexTokens>(credential.value()) {
      Ok(stored_tokens) => stored_tokens,
      Err(_) => return Err(CodexCredentialStoreError::InvalidStoredCredential),
    };
    let tokens = match stored_tokens.into_tokens() {
      Ok(tokens) => tokens,
      Err(error) => return Err(error),
    };

    return Ok(Some(tokens));
  }

  pub fn save(&self, tokens: &CodexTokens) -> Result<(), CodexCredentialStoreError> {
    let stored_tokens = match StoredCodexTokens::from_tokens(tokens) {
      Ok(stored_tokens) => stored_tokens,
      Err(error) => return Err(error),
    };
    let serialized = match serde_json::to_string(&stored_tokens) {
      Ok(serialized) => serialized,
      Err(_) => return Err(CodexCredentialStoreError::InvalidStoredCredential),
    };
    let credential = match StoredCredential::new(serialized) {
      Ok(credential) => credential,
      Err(error) => return Err(CodexCredentialStoreError::CredentialStore(error)),
    };

    match self.store.save(&self.identity, &credential) {
      Ok(()) => return Ok(()),
      Err(error) => return Err(CodexCredentialStoreError::CredentialStore(error)),
    }
  }

  pub fn delete(&self) -> Result<(), CodexCredentialStoreError> {
    match self.store.delete(&self.identity) {
      Ok(()) => return Ok(()),
      Err(error) => return Err(CodexCredentialStoreError::CredentialStore(error)),
    }
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum CodexCredentialStoreError {
  CredentialStore(CredentialStoreError),
  InvalidStoredCredential,
}

#[derive(Deserialize, Serialize)]
struct StoredCodexTokens {
  version: u8,
  access_token: String,
  refresh_token: String,
  expires_at_seconds: u64,
}

impl StoredCodexTokens {
  fn from_tokens(tokens: &CodexTokens) -> Result<Self, CodexCredentialStoreError> {
    let expires_at_seconds = match tokens.expires_at().duration_since(SystemTime::UNIX_EPOCH) {
      Ok(duration) => duration.as_secs(),
      Err(_) => return Err(CodexCredentialStoreError::InvalidStoredCredential),
    };

    return Ok(Self {
      version: 1,
      access_token: tokens.access_token().to_owned(),
      refresh_token: tokens.refresh_token().to_owned(),
      expires_at_seconds,
    });
  }

  fn into_tokens(self) -> Result<CodexTokens, CodexCredentialStoreError> {
    if self.version != 1 || self.access_token.is_empty() || self.refresh_token.is_empty() {
      return Err(CodexCredentialStoreError::InvalidStoredCredential);
    }
    let expires_at =
      match SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(self.expires_at_seconds)) {
        Some(expires_at) => expires_at,
        None => return Err(CodexCredentialStoreError::InvalidStoredCredential),
      };

    return Ok(CodexTokens::new(
      self.access_token,
      self.refresh_token,
      expires_at,
    ));
  }
}

#[cfg(test)]
mod tests {
  use super::{
    AuthorizationRequest, CallbackError, CodexCredentialStore, CodexTokens, PkceCodes, TokenClient,
    TokenExchangeError, TokenTransport, TokenTransportError, validate_callback,
  };
  use crate::security::EphemeralCredentialStore;
  use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
  use sha2::{Digest, Sha256};
  use std::{
    cell::RefCell,
    time::{Duration, SystemTime},
  };

  struct FakeTokenTransport {
    response: Result<String, TokenTransportError>,
    url: RefCell<Option<String>>,
    fields: RefCell<Vec<(String, String)>>,
  }

  impl FakeTokenTransport {
    fn succeeds(response: &str) -> Self {
      return Self {
        response: Ok(response.to_owned()),
        url: RefCell::new(None),
        fields: RefCell::new(Vec::new()),
      };
    }
  }

  impl TokenTransport for FakeTokenTransport {
    fn post_form(&self, url: &str, fields: &[(&str, &str)]) -> Result<String, TokenTransportError> {
      let mut captured_fields = Vec::new();

      for field in fields {
        let name = field.0;
        let value = field.1;

        captured_fields.push((name.to_string(), value.to_string()));
      }

      self.url.replace(Some(url.to_owned()));
      self.fields.replace(captured_fields);

      return self.response.clone();
    }
  }

  #[test]
  fn pkce_challenge_matches_the_verifier() -> () {
    let codes = PkceCodes::generate();
    let expected_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(codes.verifier().as_bytes()));

    assert_eq!(codes.challenge(), expected_challenge);

    return;
  }

  #[test]
  fn authorization_url_contains_pkce_and_state() -> () {
    let request = AuthorizationRequest::new();
    let url = request.authorization_url();

    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains(request.pkce.challenge()));
    assert!(url.contains(request.state()));

    return;
  }

  #[test]
  fn validates_a_callback_with_the_expected_state() -> () {
    let code = validate_callback("/auth/callback?code=abc&state=expected", "expected")
      .expect("the callback should be valid");

    assert_eq!(code, "abc");

    return;
  }

  #[test]
  fn rejects_a_callback_with_a_different_state() -> () {
    let error = validate_callback("/auth/callback?code=abc&state=unexpected", "expected")
      .expect_err("the callback should be rejected");

    assert_eq!(error, CallbackError::InvalidState);

    return;
  }

  #[test]
  fn exchanges_an_authorization_code_for_tokens() -> () {
    let request = AuthorizationRequest::new();
    let transport = FakeTokenTransport::succeeds(
      r#"{"access_token":"access","refresh_token":"refresh","expires_in":3600}"#,
    );
    let client = TokenClient::new(transport);
    let now = SystemTime::UNIX_EPOCH;
    let tokens = client
      .exchange_authorization_code("authorization-code", &request, now)
      .expect("the token response should be valid");

    assert_eq!(tokens.access_token(), "access");
    assert_eq!(tokens.refresh_token(), "refresh");
    assert!(!tokens.should_refresh(now));
    assert_eq!(
      client.transport.url.borrow().as_deref(),
      Some("https://auth.openai.com/oauth/token")
    );
    assert!(
      client
        .transport
        .fields
        .borrow()
        .contains(&("code".to_owned(), "authorization-code".to_owned()))
    );
    assert!(client.transport.fields.borrow().contains(&(
      "code_verifier".to_owned(),
      request.pkce.verifier().to_owned()
    )));

    return;
  }

  #[test]
  fn refreshes_tokens_with_the_refresh_grant() -> () {
    let transport = FakeTokenTransport::succeeds(
      r#"{"access_token":"next-access","refresh_token":"next-refresh","expires_in":3600}"#,
    );
    let client = TokenClient::new(transport);
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(30);
    let tokens = client
      .refresh("refresh", now)
      .expect("the token response should be valid");

    assert_eq!(tokens.access_token(), "next-access");
    assert!(
      client
        .transport
        .fields
        .borrow()
        .contains(&("grant_type".to_owned(), "refresh_token".to_owned()))
    );
    assert!(
      client
        .transport
        .fields
        .borrow()
        .contains(&("refresh_token".to_owned(), "refresh".to_owned()))
    );

    return;
  }

  #[test]
  fn rejects_an_invalid_token_response() -> () {
    let transport = FakeTokenTransport::succeeds(
      r#"{"access_token":"","refresh_token":"refresh","expires_in":0}"#,
    );
    let client = TokenClient::new(transport);
    let request = AuthorizationRequest::new();
    let result =
      client.exchange_authorization_code("authorization-code", &request, SystemTime::UNIX_EPOCH);
    let error = match result {
      Ok(_) => panic!("the token response should be rejected"),
      Err(error) => error,
    };

    assert_eq!(error, TokenExchangeError::InvalidResponse);

    return;
  }

  #[test]
  fn persists_codex_tokens_in_the_configured_store() -> () {
    let credentials = EphemeralCredentialStore::new();
    let store = CodexCredentialStore::new(credentials, "default".to_owned())
      .expect("the credential store should be configured");
    let expires_at = SystemTime::UNIX_EPOCH + Duration::from_secs(3_600);
    let tokens = CodexTokens::new("access".to_owned(), "refresh".to_owned(), expires_at);

    store.save(&tokens).expect("the tokens should be persisted");
    let loaded = store
      .load()
      .expect("the tokens should be loaded")
      .expect("the tokens should exist");

    assert_eq!(loaded.access_token(), "access");
    assert_eq!(loaded.refresh_token(), "refresh");
    assert_eq!(loaded.expires_at(), expires_at);

    store.delete().expect("the tokens should be deleted");
    let deleted = store.load().expect("the store should be available");

    assert!(deleted.is_none());

    return;
  }
}
