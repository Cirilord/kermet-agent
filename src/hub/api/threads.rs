use axum::{
  Json,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use super::super::controllers::HubState;
use crate::{ids::IdGenerator, thread::Thread, workspace::WorkspaceStoreError};

#[derive(Deserialize)]
pub(in super::super) struct CreateThreadRequest {
  workspace_id: String,
  message: String,
}

#[derive(Serialize)]
struct CreateThreadResponse {
  id: String,
}

pub(in super::super) async fn create_thread(
  State(state): State<HubState>,
  Json(request): Json<CreateThreadRequest>,
) -> Response {
  let message = request.message.trim();

  if message.is_empty() || message.len() > 4096 {
    return StatusCode::BAD_REQUEST.into_response();
  }

  let workspace = match state.workspace_store().load(&request.workspace_id) {
    Ok(workspace) => workspace,
    Err(WorkspaceStoreError::NotFound | WorkspaceStoreError::InvalidId) => {
      return StatusCode::BAD_REQUEST.into_response();
    }
    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  };
  let id = match IdGenerator::generate("thread") {
    Ok(id) => id,
    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  };
  let mut thread = Thread::new(id, workspace.path().to_path_buf(), "default".to_owned());
  thread.record_user_message(message.to_owned());

  if state.store().save(&thread).is_err() {
    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
  }
  return Json(CreateThreadResponse {
    id: thread.id().to_owned(),
  })
  .into_response();
}
