use axum::{
  body::Body,
  http::{HeaderValue, Response, header},
  middleware::Next,
  response::IntoResponse,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

use super::super::layouts::HUB_JAVASCRIPT;

#[derive(Debug)]
pub(in super::super) enum ResponseHeaderError {
  ContentSecurityPolicy,
}

pub(in super::super) async fn add_response_headers(
  request: axum::extract::Request,
  next: Next,
) -> Response<Body> {
  let response = next.run(request).await;
  let protected_response = match protect_response(response) {
    Ok(response) => response,
    Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  };

  return protected_response;
}

pub(in super::super) fn protect_response(
  mut response: Response<Body>,
) -> Result<Response<Body>, ResponseHeaderError> {
  let content_security_policy = content_security_policy();
  let content_security_policy = match HeaderValue::from_str(&content_security_policy) {
    Ok(header) => header,
    Err(_) => return Err(ResponseHeaderError::ContentSecurityPolicy),
  };
  let headers = response.headers_mut();
  headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
  headers.insert(header::CONTENT_SECURITY_POLICY, content_security_policy);
  headers.insert(
    header::REFERRER_POLICY,
    HeaderValue::from_static("no-referrer"),
  );
  headers.insert(
    header::X_CONTENT_TYPE_OPTIONS,
    HeaderValue::from_static("nosniff"),
  );
  headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
  headers.insert(
    header::HeaderName::from_static("cross-origin-opener-policy"),
    HeaderValue::from_static("same-origin"),
  );
  headers.insert(
    header::HeaderName::from_static("permissions-policy"),
    HeaderValue::from_static("camera=(), geolocation=(), microphone=()"),
  );

  return Ok(response);
}

pub(in super::super) fn content_security_policy() -> String {
  let script_hash = script_hash();
  let content_security_policy = format!(
    "default-src 'self'; base-uri 'none'; connect-src 'self' https://cdn.jsdelivr.net https://unpkg.com; font-src 'self'; form-action 'self'; frame-ancestors 'none'; img-src 'self' data:; object-src 'none'; script-src 'self' {script_hash} https://cdn.jsdelivr.net https://code.jquery.com https://unpkg.com; style-src 'self' 'unsafe-inline'; worker-src blob:"
  );

  return content_security_policy;
}

pub(in super::super) fn script_hash() -> String {
  let digest = Sha256::digest(HUB_JAVASCRIPT.as_bytes());
  let encoded_digest = STANDARD.encode(digest);
  let hash = format!("'sha256-{encoded_digest}'");

  return hash;
}
