use axum::response::Html;

pub(in super::super) async fn new_thread() -> Html<String> {
  let page = super::super::views::new_thread_page().into_string();

  return Html(page);
}
