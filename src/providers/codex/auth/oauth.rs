use std::time::Duration;

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use oauth2::{
  AuthUrl, AuthorizationCode, Client, ClientId, EndpointNotSet, EndpointSet, ExtraTokenFields,
  PkceCodeVerifier, RedirectUrl, RefreshToken, RevocationUrl, StandardRevocableToken,
  StandardTokenResponse, TokenResponse, TokenUrl,
  basic::{
    BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenType,
  },
};
use reqwest::{blocking::Client as HttpClient, redirect::Policy};
use serde::{Deserialize, Serialize};

use super::error::CodexAuthError;

const AUTHORIZATION_ENDPOINT: &str = "https://auth.openai.com/api/accounts/authorize";
const ISSUER: &str = "https://auth.openai.com";
const JWKS_URL: &str = "https://auth.openai.com/.well-known/jwks.json";
const RESOURCE: &str = "https://api.openai.com/v1";
const REVOCATION_ENDPOINT: &str = "https://auth.openai.com/api/accounts/oauth/revoke";
const TOKEN_ENDPOINT: &str = "https://auth.openai.com/api/accounts/oauth/token";

pub(super) struct IdTokenIdentity {
  pub(super) email: Option<String>,
  pub(super) subject: String,
}

pub(super) struct TokenSet {
  pub(super) access_token: String,
  pub(super) expires_in: u64,
  pub(super) id_token: String,
  pub(super) refresh_token: String,
  pub(super) scopes: Vec<String>,
}

pub(super) struct OAuthClient {
  http: HttpClient,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct OpenAiExtraTokenFields {
  id_token: String,
}

impl ExtraTokenFields for OpenAiExtraTokenFields {}

type OpenAiTokenResponse = StandardTokenResponse<OpenAiExtraTokenFields, BasicTokenType>;
type BaseOpenAiClient = Client<
  BasicErrorResponse,
  OpenAiTokenResponse,
  BasicTokenIntrospectionResponse,
  StandardRevocableToken,
  BasicRevocationErrorResponse,
>;
type OpenAiClient = Client<
  BasicErrorResponse,
  OpenAiTokenResponse,
  BasicTokenIntrospectionResponse,
  StandardRevocableToken,
  BasicRevocationErrorResponse,
  EndpointSet,
  EndpointNotSet,
  EndpointNotSet,
  EndpointNotSet,
  EndpointSet,
>;

impl OAuthClient {
  pub(super) fn new() -> Result<OAuthClient, CodexAuthError> {
    let result = HttpClient::builder()
      .redirect(Policy::none())
      .timeout(Duration::from_secs(15))
      .build();
    let http = match result {
      Ok(http) => http,
      Err(_) => return Err(CodexAuthError::TokenExchange),
    };
    let client = OAuthClient { http };

    return Ok(client);
  }

  pub(super) fn exchange_code(
    &self,
    client_id: &str,
    code: &str,
    code_verifier: PkceCodeVerifier,
    redirect_uri: &str,
  ) -> Result<TokenSet, CodexAuthError> {
    let client = match create_client(client_id, redirect_uri) {
      Ok(client) => client,
      Err(error) => return Err(error),
    };
    let request = client
      .exchange_code(AuthorizationCode::new(code.to_owned()))
      .set_pkce_verifier(code_verifier)
      .add_extra_param("resource", RESOURCE);
    let response = match request.request(&self.http) {
      Ok(response) => response,
      Err(_) => return Err(CodexAuthError::TokenExchange),
    };
    let refresh_token = match response.refresh_token() {
      Some(refresh_token) => refresh_token.secret().to_owned(),
      None => return Err(CodexAuthError::TokenExchange),
    };
    let expires_in = match response.expires_in() {
      Some(expires_in) => expires_in.as_secs(),
      None => return Err(CodexAuthError::TokenExchange),
    };
    let response_scopes = match response.scopes() {
      Some(scopes) => scopes,
      None => return Err(CodexAuthError::TokenExchange),
    };
    let mut scopes = Vec::new();

    for scope in response_scopes {
      scopes.push(scope.to_string());
    }
    if *response.token_type() != BasicTokenType::Bearer {
      return Err(CodexAuthError::TokenExchange);
    }
    let token_set = TokenSet {
      access_token: response.access_token().secret().to_owned(),
      expires_in,
      id_token: response.extra_fields().id_token.clone(),
      refresh_token,
      scopes,
    };

    return Ok(token_set);
  }

  pub(super) fn validate_id_token(
    &self,
    id_token: &str,
    client_id: &str,
    nonce: &str,
  ) -> Result<IdTokenIdentity, CodexAuthError> {
    let header = match decode_header(id_token) {
      Ok(header) => header,
      Err(_) => return Err(CodexAuthError::IdentityValidation),
    };
    let key_id = match header.kid {
      Some(key_id) => key_id,
      None => return Err(CodexAuthError::IdentityValidation),
    };
    let result = self.http.get(JWKS_URL).send();
    let jwks_response = match result {
      Ok(jwks_response) => jwks_response,
      Err(_) => return Err(CodexAuthError::IdentityValidation),
    };

    if !jwks_response.status().is_success() {
      return Err(CodexAuthError::IdentityValidation);
    }
    let jwks: JwkSet = match jwks_response.json() {
      Ok(jwks) => jwks,
      Err(_) => return Err(CodexAuthError::IdentityValidation),
    };
    let jwk = match jwks.find(&key_id) {
      Some(jwk) => jwk,
      None => return Err(CodexAuthError::IdentityValidation),
    };
    let decoding_key = match DecodingKey::from_jwk(jwk) {
      Ok(decoding_key) => decoding_key,
      Err(_) => return Err(CodexAuthError::IdentityValidation),
    };
    let mut validation = Validation::new(Algorithm::RS256);

    validation.set_audience(&[client_id]);
    validation.set_issuer(&[ISSUER]);
    validation.set_required_spec_claims(&["aud", "exp", "iss", "sub"]);

    let data = match decode::<IdTokenClaims>(id_token, &decoding_key, &validation) {
      Ok(data) => data,
      Err(_) => return Err(CodexAuthError::IdentityValidation),
    };

    if data.claims.nonce != nonce || data.claims.subject.is_empty() {
      return Err(CodexAuthError::IdentityValidation);
    }
    let identity = IdTokenIdentity {
      email: data.claims.email,
      subject: data.claims.subject,
    };

    return Ok(identity);
  }

  pub(super) fn revoke_refresh_token(&self, client_id: &str, refresh_token: &str) -> bool {
    let revocation_url = match RevocationUrl::new(REVOCATION_ENDPOINT.to_owned()) {
      Ok(revocation_url) => revocation_url,
      Err(_) => return false,
    };
    let client =
      BaseOpenAiClient::new(ClientId::new(client_id.to_owned())).set_revocation_url(revocation_url);
    let token = RefreshToken::new(refresh_token.to_owned());
    let request = match client.revoke_token(token.into()) {
      Ok(request) => request,
      Err(_) => return false,
    };
    let result = request.request(&self.http);

    return result.is_ok();
  }
}

pub(super) fn create_client(
  client_id: &str,
  redirect_uri: &str,
) -> Result<OpenAiClient, CodexAuthError> {
  let authorization_url = match AuthUrl::new(AUTHORIZATION_ENDPOINT.to_owned()) {
    Ok(authorization_url) => authorization_url,
    Err(_) => return Err(CodexAuthError::AuthorizationRequest),
  };
  let token_url = match TokenUrl::new(TOKEN_ENDPOINT.to_owned()) {
    Ok(token_url) => token_url,
    Err(_) => return Err(CodexAuthError::AuthorizationRequest),
  };
  let redirect_url = match RedirectUrl::new(redirect_uri.to_owned()) {
    Ok(redirect_url) => redirect_url,
    Err(_) => return Err(CodexAuthError::AuthorizationRequest),
  };
  let client = BaseOpenAiClient::new(ClientId::new(client_id.to_owned()))
    .set_auth_uri(authorization_url)
    .set_token_uri(token_url)
    .set_redirect_uri(redirect_url);

  return Ok(client);
}

#[derive(Deserialize)]
struct IdTokenClaims {
  email: Option<String>,
  nonce: String,
  #[serde(rename = "sub")]
  subject: String,
}
