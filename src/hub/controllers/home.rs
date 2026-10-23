use axum::response::Html;

pub(in super::super) async fn home() -> Html<String> {
  let page = super::super::views::home_page().into_string();

  return Html(page);
}
