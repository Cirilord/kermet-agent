mod bootstrap;
mod home;
mod list_threads;
mod new_thread;
mod show_thread;

use crate::thread::ThreadStore;
use axum::{response::IntoResponse, response::Response};
use std::{path::PathBuf, sync::Arc};

use super::security::auth::HubAccess;

pub(super) use bootstrap::bootstrap;
pub(super) use home::home;
pub(super) use list_threads::list_threads;
pub(super) use new_thread::new_thread;
pub(super) use show_thread::show_thread;

#[derive(Clone)]
pub(super) struct HubState {
  store: Arc<ThreadStore>,
  access: Arc<HubAccess>,
}

impl HubState {
  pub(super) fn new(directory: PathBuf) -> Self {
    let state = Self {
      store: Arc::new(ThreadStore::new(directory)),
      access: Arc::new(HubAccess::new()),
    };

    return state;
  }

  pub(super) fn access(&self) -> &HubAccess {
    return &self.access;
  }
}

pub(super) enum HubResponseError {
  NotFound,
  ThreadStore,
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
    };
    let page = super::views::error_page(title, message).into_string();
    let response = (status, axum::response::Html(page)).into_response();

    return response;
  }
}
