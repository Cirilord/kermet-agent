use crate::thread::ThreadStoreError;
use axum::{
  extract::{Path, State},
  response::Html,
};

use super::{HubResponseError, HubState};

pub(in super::super) async fn show_thread(
  Path(id): Path<String>,
  State(state): State<HubState>,
) -> Result<Html<String>, HubResponseError> {
  let thread = match state.store.load(&id) {
    Ok(thread) => thread,
    Err(ThreadStoreError::NotFound) => return Err(HubResponseError::NotFound),
    Err(_) => return Err(HubResponseError::ThreadStore),
  };
  let threads = match state.store().list() {
    Ok(threads) => threads,
    Err(_) => return Err(HubResponseError::ThreadStore),
  };
  let page =
    super::super::views::thread_detail_page(&thread, &threads, state.access().csrf_token())
      .into_string();

  return Ok(Html(page));
}
