use axum::{extract::State, response::Html};

use super::{HubResponseError, HubState};

pub(in super::super) async fn list_threads(
  State(state): State<HubState>,
) -> Result<Html<String>, HubResponseError> {
  let threads = match state.store.list() {
    Ok(threads) => threads,
    Err(_) => return Err(HubResponseError::ThreadStore),
  };
  let page = super::super::views::thread_list_page(&threads).into_string();

  return Ok(Html(page));
}
