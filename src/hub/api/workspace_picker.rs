use axum::{
  Json,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf, process::Stdio, time::Duration};
use tokio::{process::Command, time::timeout};

use super::super::controllers::HubState;
use crate::{
  ids::IdGenerator,
  workspace::{WorkspaceRecord, WorkspaceStore},
};

const PICKER_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_PICKER_OUTPUT_BYTES: usize = 16 * 1024;

#[derive(Deserialize, Serialize)]
struct WorkspacePickerOutput {
  path: Option<String>,
}

async fn run_workspace_picker() -> Result<WorkspacePickerOutput, ()> {
  let executable = match env::current_exe() {
    Ok(executable) => executable,
    Err(_) => return Err(()),
  };
  let mut command = Command::new(executable);
  command
    .args(["hub", "--open-picker"])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .kill_on_drop(true);
  let child = match command.spawn() {
    Ok(child) => child,
    Err(_) => return Err(()),
  };
  let output = match timeout(PICKER_TIMEOUT, child.wait_with_output()).await {
    Ok(Ok(output)) => output,
    Ok(Err(_)) | Err(_) => return Err(()),
  };

  if !output.status.success() || output.stdout.len() > MAX_PICKER_OUTPUT_BYTES {
    return Err(());
  }

  let output = match serde_json::from_slice::<WorkspacePickerOutput>(&output.stdout) {
    Ok(output) => output,
    Err(_) => return Err(()),
  };

  return Ok(output);
}

fn save_workspace(store: &WorkspaceStore, path: PathBuf) -> Result<WorkspaceRecord, ()> {
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
  let workspaces = match store.list() {
    Ok(workspaces) => workspaces,
    Err(_) => return Err(()),
  };

  for workspace in workspaces {
    if workspace.path() == path {
      return Ok(workspace);
    }
  }

  let id = match IdGenerator::generate("workspace") {
    Ok(id) => id,
    Err(_) => return Err(()),
  };
  let workspace = WorkspaceRecord::new(id, path);

  if store.save(&workspace).is_err() {
    return Err(());
  }

  return Ok(workspace);
}

pub(in super::super) async fn open_workspace_picker(State(state): State<HubState>) -> Response {
  let output = match run_workspace_picker().await {
    Ok(output) => output,
    Err(()) => {
      return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
  };

  let path = match output.path {
    Some(path) => path,
    None => {
      return StatusCode::NO_CONTENT.into_response();
    }
  };

  let workspace = match save_workspace(state.workspace_store(), PathBuf::from(path)) {
    Ok(workspace) => workspace,
    Err(()) => {
      return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
  };

  return Json(workspace).into_response();
}
