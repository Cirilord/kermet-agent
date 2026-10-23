use super::layouts::HUB_JAVASCRIPT;
use super::views;
use crate::thread::Thread;
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
  assert!(page.contains("@tailwindcss/browser@4.3.3"));
  assert!(page.contains("jquery-4.0.0.min.js"));
  assert!(page.contains("lucide@1.61.0/dist/umd/lucide.min.js"));

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
  assert!(page.contains(HUB_JAVASCRIPT));

  return;
}
