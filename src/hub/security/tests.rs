use super::{
  auth::{AccessDecision, HubAccess, RouteAccess, SessionTransport},
  request_limits::request_limit_violation,
  response_headers::{content_security_policy, protect_response, script_hash},
};
use axum::{
  body::Body,
  http::{HeaderMap, HeaderValue, Response, header},
};

#[test]
fn accepts_a_bootstrap_token_only_once() -> () {
  let access = HubAccess::new();
  let token = access.bootstrap_token().to_owned();

  assert!(!access.consume_bootstrap_token("invalid"));
  assert!(access.consume_bootstrap_token(&token));
  assert!(!access.consume_bootstrap_token(&token));

  return;
}

#[test]
fn accepts_only_the_issued_session_cookie() -> () {
  let access = HubAccess::new();
  let cookie = access.session_cookie(SessionTransport::Http);
  let mut valid_headers = HeaderMap::new();
  let valid_cookie = HeaderValue::from_str(&cookie).expect("the generated cookie should be valid");
  valid_headers.insert(header::COOKIE, valid_cookie);
  let mut invalid_headers = HeaderMap::new();
  invalid_headers.insert(
    header::COOKIE,
    HeaderValue::from_static("kermet_hub_session=invalid"),
  );

  assert!(access.is_valid_session(&valid_headers));
  assert!(!access.is_valid_session(&invalid_headers));
  assert!(!access.is_valid_session(&HeaderMap::new()));

  return;
}

#[test]
fn keeps_the_generic_mutation_boundary_fail_closed() -> () {
  let access = HubAccess::new();
  let cookie = access.session_cookie(SessionTransport::Http);
  let mut headers = HeaderMap::new();
  let cookie = HeaderValue::from_str(&cookie).expect("the generated cookie should be valid");
  headers.insert(header::COOKIE, cookie);

  assert_eq!(
    access.authorize(&headers, RouteAccess::Read),
    AccessDecision::Allowed
  );
  assert_eq!(
    access.authorize(&headers, RouteAccess::Mutation),
    AccessDecision::MutationProtectionRequired
  );

  return;
}

#[test]
fn requires_session_origin_and_csrf_token_for_mutations() -> () {
  let access = HubAccess::new();
  let cookie = access.session_cookie(SessionTransport::Http);
  let mut headers = HeaderMap::new();
  let cookie = HeaderValue::from_str(&cookie).expect("the generated cookie should be valid");
  headers.insert(header::COOKIE, cookie);
  headers.insert(
    header::ORIGIN,
    HeaderValue::from_static("http://127.0.0.1:3000"),
  );
  let csrf_token =
    HeaderValue::from_str(access.csrf_token()).expect("the generated CSRF token should be valid");
  headers.insert("x-kermet-csrf", csrf_token);

  assert_eq!(
    access.authorize_mutation(&headers, "http://127.0.0.1:3000"),
    AccessDecision::Allowed
  );
  assert_eq!(
    access.authorize_mutation(&headers, "http://localhost:3000"),
    AccessDecision::InvalidOrigin
  );
  headers.insert("x-kermet-csrf", HeaderValue::from_static("invalid"));
  assert_eq!(
    access.authorize_mutation(&headers, "http://127.0.0.1:3000"),
    AccessDecision::InvalidCsrfToken
  );

  return;
}

#[test]
fn adds_the_secure_attribute_to_https_session_cookies() -> () {
  let access = HubAccess::new();
  let cookie = access.session_cookie(SessionTransport::Https);

  assert!(cookie.contains("HttpOnly"));
  assert!(cookie.contains("SameSite=Strict"));
  assert!(cookie.contains("Secure"));

  return;
}

#[test]
fn adds_protection_headers_to_hub_responses() -> () {
  let response = protect_response(Response::new(Body::empty()))
    .expect("the generated content-security policy should be a valid header");

  assert_eq!(
    response.headers().get(header::CACHE_CONTROL),
    Some(&HeaderValue::from_static("no-store"))
  );
  assert_eq!(
    response.headers().get(header::REFERRER_POLICY),
    Some(&HeaderValue::from_static("no-referrer"))
  );
  assert_eq!(
    response.headers().get(header::X_CONTENT_TYPE_OPTIONS),
    Some(&HeaderValue::from_static("nosniff"))
  );
  assert_eq!(
    response.headers().get(header::X_FRAME_OPTIONS),
    Some(&HeaderValue::from_static("DENY"))
  );
  assert!(
    response
      .headers()
      .contains_key(header::CONTENT_SECURITY_POLICY)
  );
  let content_security_policy = content_security_policy();

  assert!(content_security_policy.contains(&script_hash()));
  assert!(
    content_security_policy
      .contains("connect-src 'self' https://cdn.jsdelivr.net https://unpkg.com")
  );

  return;
}

#[test]
fn rejects_requests_with_bodies() -> () {
  let request = axum::http::Request::builder()
    .header(header::CONTENT_LENGTH, "1")
    .body(Body::empty())
    .expect("the test request should be valid");

  assert_eq!(
    request_limit_violation(&request),
    Some(axum::http::StatusCode::PAYLOAD_TOO_LARGE)
  );

  return;
}

#[test]
fn accepts_a_bounded_thread_message_body() -> () {
  let request = axum::http::Request::builder()
    .method(axum::http::Method::POST)
    .uri("/api/threads/thread-0123456789abcdef/messages")
    .header(header::CONTENT_LENGTH, "1")
    .body(Body::empty())
    .expect("the test request should be valid");

  assert_eq!(request_limit_violation(&request), None);

  return;
}

#[test]
fn rejects_requests_with_a_long_target() -> () {
  let request_target = format!("/?{}", "a".repeat(2 * 1024));
  let request = axum::http::Request::builder()
    .uri(request_target)
    .body(Body::empty())
    .expect("the test request should be valid");

  assert_eq!(
    request_limit_violation(&request),
    Some(axum::http::StatusCode::URI_TOO_LONG)
  );

  return;
}

#[test]
fn rejects_requests_with_large_headers() -> () {
  let request = axum::http::Request::builder()
    .header("x-padding", "a".repeat(8 * 1024))
    .body(Body::empty())
    .expect("the test request should be valid");

  assert_eq!(
    request_limit_violation(&request),
    Some(axum::http::StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)
  );

  return;
}
