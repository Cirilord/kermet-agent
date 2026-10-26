mod bootstrap;
mod home;
mod list_threads;
mod new_thread;
mod show_thread;

use crate::{thread::ThreadStore, workspace::WorkspaceStore};
use axum::{response::IntoResponse, response::Response};
use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

use super::security::auth::HubAccess;

pub(super) use bootstrap::bootstrap;
pub(super) use home::home;
pub(super) use list_threads::list_threads;
pub(super) use new_thread::new_thread;
pub(super) use show_thread::show_thread;

#[derive(Clone)]
pub(super) struct HubState {
  store: Arc<ThreadStore>,
  workspace_store: Arc<WorkspaceStore>,
  access: Arc<HubAccess>,
  origin: String,
  events: Arc<Mutex<std::collections::HashMap<String, broadcast::Sender<HubEvent>>>>,
}

#[derive(Clone)]
pub(super) struct HubEvent {
  pub(super) name: String,
  pub(super) data: String,
}

impl HubState {
  pub(super) fn new(
    thread_directory: PathBuf,
    workspace_directory: PathBuf,
    origin: String,
  ) -> Self {
    let state = Self {
      store: Arc::new(ThreadStore::new(thread_directory)),
      workspace_store: Arc::new(WorkspaceStore::new(workspace_directory)),
      access: Arc::new(HubAccess::new()),
      origin,
      events: Arc::new(Mutex::new(std::collections::HashMap::new())),
    };

    return state;
  }

  pub(super) fn access(&self) -> &HubAccess {
    return &self.access;
  }

  pub(super) fn workspace_store(&self) -> &WorkspaceStore {
    return &self.workspace_store;
  }

  pub(super) fn store(&self) -> &ThreadStore {
    return &self.store;
  }

  pub(super) fn origin(&self) -> &str {
    return &self.origin;
  }

  pub(super) fn subscribe(&self, id: &str) -> broadcast::Receiver<HubEvent> {
    let mut events = match self.events.lock() {
      Ok(events) => events,
      Err(_) => return broadcast::channel(1).1,
    };
    let sender = events.entry(id.to_owned()).or_insert_with(|| {
      let (sender, _) = broadcast::channel(64);
      return sender;
    });
    let receiver = sender.subscribe();
    return receiver;
  }

  pub(super) fn publish(&self, id: &str, name: &str, data: String) -> () {
    let mut events = match self.events.lock() {
      Ok(events) => events,
      Err(_) => return,
    };
    let sender = events.entry(id.to_owned()).or_insert_with(|| {
      let (sender, _) = broadcast::channel(64);
      return sender;
    });
    let event = HubEvent {
      name: name.to_owned(),
      data,
    };
    let _: Result<usize, broadcast::error::SendError<HubEvent>> = sender.send(event);

    return;
  }
}

pub(super) enum HubResponseError {
  NotFound,
  ThreadStore,
  WorkspaceStore,
}

impl IntoResponse for HubResponseError {
  fn into_response(self) -> Response {
    let (status, title, message) = match self {
      Self::NotFound => (
        axum::http::StatusCode::NOT_FOUND,
        "Thread not found",
        "The requested thread does not exist.",
      ),
      Self::ThreadStore => (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "Thread store error",
        "Kermet could not read the thread store.",
      ),
      Self::WorkspaceStore => (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "Workspace store error",
        "Kermet could not read the workspace store.",
      ),
    };
    let page = super::views::error_page(title, message).into_string();
    let response = (status, axum::response::Html(page)).into_response();

    return response;
  }
}
