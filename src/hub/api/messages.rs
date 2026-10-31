use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::path::PathBuf;

use super::super::controllers::HubState;
use crate::{
  agent::{AgentEvent, TaskRequest},
  config::{ModelOverrides, resolve_model_selection, user_path},
  security::{
    ApprovalPolicy, ExecutionPolicy,
    sandbox::{NetworkAccess, SandboxMode},
  },
  turn::execute_turn,
  workspace::Workspace,
};

#[derive(Deserialize)]
pub(in super::super) struct CreateMessageRequest {
  message: String,
}

pub(in super::super) async fn create_message(
  Path(id): Path<String>,
  State(state): State<HubState>,
  Json(request): Json<CreateMessageRequest>,
) -> Response {
  let message = request.message.trim().to_owned();
  if message.is_empty() || message.len() > 4096 {
    return StatusCode::BAD_REQUEST.into_response();
  }
  let mut thread = match state.store().load(&id) {
    Ok(thread) => thread,
    Err(_) => return StatusCode::NOT_FOUND.into_response(),
  };
  thread.record_user_message(message.clone());
  if state.store().save(&thread).is_err() {
    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
  }

  start_turn(state, thread, message);

  return StatusCode::ACCEPTED.into_response();
}

pub(in super::super) async fn start_initial_turn(
  Path(id): Path<String>,
  State(state): State<HubState>,
) -> Response {
  let thread = match state.store().load(&id) {
    Ok(thread) => thread,
    Err(_) => return StatusCode::NOT_FOUND.into_response(),
  };
  let message = match thread.latest_user_message() {
    Some(message) => message.to_owned(),
    None => return StatusCode::BAD_REQUEST.into_response(),
  };

  start_turn(state, thread, message);

  return StatusCode::ACCEPTED.into_response();
}

fn start_turn(state: HubState, thread: crate::thread::Thread, message: String) -> () {
  let thread_id = thread.id().to_owned();
  let workspace_path = thread.workspace().to_path_buf();
  let profile = thread.profile().to_owned();
  let state_for_turn = state.clone();
  tokio::task::spawn_blocking(move || {
    let config_path = match user_path() {
      Ok(path) => path,
      Err(_) => {
        state_for_turn.publish(&thread_id, "failed", "configuration".to_owned());
        return;
      }
    };
    let selection = match resolve_model_selection(&config_path, &profile, ModelOverrides::none()) {
      Ok(selection) => selection,
      Err(_) => {
        state_for_turn.publish(&thread_id, "failed", "configuration".to_owned());
        return;
      }
    };
    let workspace = match open_workspace(workspace_path) {
      Ok(workspace) => workspace,
      Err(()) => {
        state_for_turn.publish(&thread_id, "failed", "workspace".to_owned());
        return;
      }
    };
    let execution_policy = match execution_policy() {
      Ok(policy) => policy,
      Err(_) => {
        state_for_turn.publish(&thread_id, "failed", "policy".to_owned());
        return;
      }
    };
    let request = match TaskRequest::new(message, workspace, execution_policy) {
      Ok(request) => request,
      Err(_) => {
        state_for_turn.publish(&thread_id, "failed", "request".to_owned());
        return;
      }
    };
    let mut on_event = |event| match event {
      AgentEvent::Model(crate::models::ModelEvent::ThinkingStarted) => {
        state_for_turn.publish(&thread_id, "thinking", "started".to_owned())
      }
      AgentEvent::Model(crate::models::ModelEvent::TextDelta(text)) => {
        state_for_turn.publish(&thread_id, "text_delta", text)
      }
      AgentEvent::ToolCall { tool_name, target } => state_for_turn.publish(
        &thread_id,
        "tool_call",
        format!("{}:{target}", tool_name.function_name()),
      ),
      AgentEvent::ToolExecution { tool_name, target } => state_for_turn.publish(
        &thread_id,
        "tool_execution",
        format!("{}:{target}", tool_name.function_name()),
      ),
    };
    let answer = match execute_turn(&request, &selection, &profile, &mut on_event) {
      Ok(answer) => answer,
      Err(_) => {
        state_for_turn.publish(&thread_id, "failed", "execution".to_owned());
        return;
      }
    };
    let mut thread = match state_for_turn.store().load(&thread_id) {
      Ok(thread) => thread,
      Err(_) => return,
    };
    thread.record_assistant_message(answer);
    if state_for_turn.store().save(&thread).is_ok() {
      state_for_turn.publish(&thread_id, "completed", "completed".to_owned());
    } else {
      state_for_turn.publish(&thread_id, "failed", "persistence".to_owned());
    }
  });

  return;
}

fn execution_policy() -> Result<ExecutionPolicy, crate::security::ExecutionPolicyError> {
  let policy = ExecutionPolicy::new(
    SandboxMode::DangerFullAccess,
    NetworkAccess::Allowed,
    ApprovalPolicy::Never,
  );

  return policy;
}

fn open_workspace(path: PathBuf) -> Result<Workspace, ()> {
  let metadata = match std::fs::metadata(&path) {
    Ok(metadata) => metadata,
    Err(_) => return Err(()),
  };
  if !metadata.is_dir() {
    return Err(());
  }
  let path = match std::fs::canonicalize(path) {
    Ok(path) => path,
    Err(_) => return Err(()),
  };
  return Ok(Workspace::new(path.clone(), path.join(".git").exists()));
}

#[cfg(test)]
mod tests {
  use super::execution_policy;
  use crate::security::{
    ApprovalPolicy, ExecutionPolicy,
    sandbox::{NetworkAccess, SandboxMode},
  };

  #[test]
  fn enables_full_access_for_hub_turns() -> () {
    let expected = ExecutionPolicy::new(
      SandboxMode::DangerFullAccess,
      NetworkAccess::Allowed,
      ApprovalPolicy::Never,
    );

    assert_eq!(execution_policy(), expected);

    return;
  }
}
