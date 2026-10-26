use axum::http::{HeaderMap, header};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};

const SESSION_COOKIE_NAME: &str = "kermet_hub_session";

#[derive(Debug, Eq, PartialEq)]
pub(in super::super) enum AccessDecision {
  Allowed,
  Unauthenticated,
  MutationProtectionRequired,
  InvalidOrigin,
  InvalidCsrfToken,
}

pub(in super::super) enum RouteAccess {
  Read,
  #[allow(
    dead_code,
    reason = "Mutation routes use authorize_mutation because they require origin and CSRF validation."
  )]
  Mutation,
}

pub(in super::super) enum SessionTransport {
  Http,
  #[allow(
    dead_code,
    reason = "The Hub currently binds only to local HTTP and does not serve HTTPS."
  )]
  Https,
}

pub(in super::super) struct HubAccess {
  bootstrap_token: String,
  session_token: String,
  csrf_token: String,
  bootstrap_consumed: AtomicBool,
}

impl HubAccess {
  pub(in super::super) fn new() -> Self {
    let bootstrap_token = generate_token();
    let session_token = generate_token();
    let csrf_token = generate_token();

    return Self {
      bootstrap_token,
      session_token,
      csrf_token,
      bootstrap_consumed: AtomicBool::new(false),
    };
  }

  pub(in super::super) fn bootstrap_token(&self) -> &str {
    return &self.bootstrap_token;
  }

  pub(in super::super) fn csrf_token(&self) -> &str {
    return &self.csrf_token;
  }

  pub(in super::super) fn consume_bootstrap_token(&self, candidate: &str) -> bool {
    if candidate != self.bootstrap_token {
      return false;
    }

    let consumed =
      self
        .bootstrap_consumed
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire);
    let accepted = consumed.is_ok();

    return accepted;
  }

  pub(in super::super) fn is_valid_session(&self, headers: &HeaderMap) -> bool {
    let session_token = match session_token(headers) {
      Some(session_token) => session_token,
      None => return false,
    };
    let valid = session_token == self.session_token;

    return valid;
  }

  pub(in super::super) fn authorize(
    &self,
    headers: &HeaderMap,
    route_access: RouteAccess,
  ) -> AccessDecision {
    if !self.is_valid_session(headers) {
      return AccessDecision::Unauthenticated;
    }

    let decision = match route_access {
      RouteAccess::Read => AccessDecision::Allowed,
      RouteAccess::Mutation => AccessDecision::MutationProtectionRequired,
    };

    return decision;
  }

  pub(in super::super) fn authorize_mutation(
    &self,
    headers: &HeaderMap,
    expected_origin: &str,
  ) -> AccessDecision {
    if !self.is_valid_session(headers) {
      return AccessDecision::Unauthenticated;
    }
    let origin = match headers.get(header::ORIGIN) {
      Some(origin) => origin,
      None => return AccessDecision::InvalidOrigin,
    };
    let origin = match origin.to_str() {
      Ok(origin) => origin,
      Err(_) => return AccessDecision::InvalidOrigin,
    };

    if origin != expected_origin {
      return AccessDecision::InvalidOrigin;
    }

    let csrf_token = match headers.get("x-kermet-csrf") {
      Some(csrf_token) => csrf_token,
      None => return AccessDecision::InvalidCsrfToken,
    };
    let csrf_token = match csrf_token.to_str() {
      Ok(csrf_token) => csrf_token,
      Err(_) => return AccessDecision::InvalidCsrfToken,
    };

    if csrf_token != self.csrf_token {
      return AccessDecision::InvalidCsrfToken;
    }

    return AccessDecision::Allowed;
  }

  pub(in super::super) fn session_cookie(&self, transport: SessionTransport) -> String {
    let cookie = match transport {
      SessionTransport::Http => format!(
        "{SESSION_COOKIE_NAME}={}; HttpOnly; SameSite=Strict; Path=/",
        self.session_token
      ),
      SessionTransport::Https => format!(
        "{SESSION_COOKIE_NAME}={}; HttpOnly; SameSite=Strict; Secure; Path=/",
        self.session_token
      ),
    };

    return cookie;
  }
}

fn generate_token() -> String {
  let mut random_bytes = [0_u8; 32];
  rand::rng().fill(&mut random_bytes);
  let token = URL_SAFE_NO_PAD.encode(random_bytes);

  return token;
}

fn session_token(headers: &HeaderMap) -> Option<&str> {
  let cookie_header = match headers.get(header::COOKIE) {
    Some(cookie_header) => cookie_header,
    None => return None,
  };
  let cookie_header = match cookie_header.to_str() {
    Ok(cookie_header) => cookie_header,
    Err(_) => return None,
  };

  for entry in cookie_header.split(';') {
    let entry = entry.trim();
    let (name, value) = match entry.split_once('=') {
      Some(pair) => pair,
      None => continue,
    };

    if name == SESSION_COOKIE_NAME {
      return Some(value);
    }
  }

  return None;
}
