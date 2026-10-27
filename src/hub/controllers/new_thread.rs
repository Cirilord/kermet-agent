use axum::{extract::State, response::Html};

use super::{HubResponseError, HubState};

pub(in super::super) async fn new_thread(
  State(state): State<HubState>,
) -> Result<Html<String>, HubResponseError> {
  let workspaces = match state.workspace_store().list() {
    Ok(workspaces) => workspaces,
    Err(_) => return Err(HubResponseError::WorkspaceStore),
  };
  let threads = match state.store().list() {
    Ok(threads) => threads,
    Err(_) => return Err(HubResponseError::ThreadStore),
  };
  let page =
    super::super::views::new_thread_page(&workspaces, &threads, state.access().csrf_token())
      .into_string();

  return Ok(Html(page));
}
