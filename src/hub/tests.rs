use super::layouts::HUB_JAVASCRIPT;
use super::views;
use crate::thread::Thread;
use crate::workspace::WorkspaceRecord;
use std::path::PathBuf;

#[test]
fn renders_thread_content_as_escaped_html() -> () {
  let mut thread = Thread::new(
    "thread-0123456789abcdef".to_owned(),
    PathBuf::from("/workspace"),
    "default".to_owned(),
  );
  thread.record_user_message("<script>unsafe()</script>".to_owned());
  let page = views::thread_detail_page(&thread, "csrf-token").into_string();

  assert!(page.contains("&lt;script&gt;unsafe()&lt;/script&gt;"));
  assert!(!page.contains("<script>unsafe()</script>"));
  assert!(page.contains("id=\"thread-transcript\""));
  assert!(page.contains("id=\"new-thread-form\""));
  assert!(!page.contains("<pre"));

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
fn renders_the_new_thread_chat() -> () {
  let workspaces = vec![WorkspaceRecord::new(
    "workspace-0123456789abcdef".to_owned(),
    PathBuf::from("/workspace"),
  )];
  let page = views::new_thread_page(&workspaces, "csrf-token").into_string();

  assert!(page.contains(">Start a new thread</h1>"));
  assert!(page.contains("id=\"thread-message\""));
  assert!(page.contains("autofocus"));
  assert!(page.contains("disabled"));
  assert!(page.contains("id=\"workspace-trigger\""));
  assert!(page.contains("/workspace"));
  assert!(page.contains("id=\"new-workspace-trigger\""));
  assert!(page.contains("id=\"workspace-dialog\""));
  assert!(page.contains("id=\"workspace-csrf-token\""));
  assert!(page.contains("id=\"workspace-picker-trigger\""));
  assert!(page.contains("id=\"workspace-picker-status\""));

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
  let page = views::thread_detail_page(&thread, "csrf-token").into_string();

  assert!(page.contains("id=\"hub-sidebar\""));
  assert!(page.contains(">Platform</p>"));
  assert!(page.contains("id=\"sidebar-toggle\""));
  assert!(page.contains("data-lucide=\"panel-left\""));
  assert!(page.contains("data-lucide=\"messages-square\""));
  assert!(page.contains("aria-controls=\"threads-menu\""));
  assert!(page.contains("id=\"threads-menu\""));
  assert!(page.contains("href=\"/threads/new\""));
  assert!(page.contains(">New thread</a>"));
  assert!(page.contains(HUB_JAVASCRIPT));

  return;
}
