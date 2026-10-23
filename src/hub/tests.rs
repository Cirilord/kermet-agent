use super::auth::HubAccess;
use super::views;
use crate::thread::Thread;
use axum::http::{HeaderMap, HeaderValue, header};
use std::path::PathBuf;

#[test]
fn renders_thread_content_as_escaped_html() -> () {
  let mut thread = Thread::new(
    "thread-0123456789abcdef".to_owned(),
    PathBuf::from("/workspace"),
    "default".to_owned(),
  );
  thread.record_user_message("<script>unsafe()</script>".to_owned());
  let page = views::thread_detail_page(&thread).into_string();

  assert!(page.contains("&lt;script&gt;unsafe()&lt;/script&gt;"));
  assert!(!page.contains("<script>unsafe()</script>"));

  return;
}

#[test]
fn renders_the_home_page() -> () {
  let page = views::home_page().into_string();

  assert!(page.contains(">Hello</h1>"));
  assert!(page.contains(">Kermet Hub</a>"));
  assert!(!page.contains(">Home</span>"));

  return;
}

#[test]
fn renders_the_application_sidebar() -> () {
  let mut thread = Thread::new(
    "thread-0123456789abcdef".to_owned(),
    PathBuf::from("/workspace"),
    "default".to_owned(),
  );
  thread.record_user_message("Hello".to_owned());
  let page = views::thread_detail_page(&thread).into_string();

  assert!(page.contains("id=\"hub-sidebar\""));
  assert!(page.contains(">Platform</p>"));
  assert!(page.contains("id=\"sidebar-toggle\""));
  assert!(page.contains("data-lucide=\"panel-left\""));
  assert!(page.contains("data-lucide=\"messages-square\""));
  assert!(page.contains("aria-controls=\"threads-menu\""));
  assert!(page.contains("id=\"threads-menu\""));

  return;
}

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
  let cookie = access.session_cookie();
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
