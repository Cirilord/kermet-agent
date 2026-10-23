use crate::thread::{ThreadStore, ThreadStoreError};
use axum::{
  Router,
  extract::{Path, State},
  http::StatusCode,
  response::{Html, IntoResponse, Response},
  routing::get,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use super::views;

#[derive(Debug)]
pub enum HubError {
  Runtime,
  Bind,
  Address,
  Serve,
}

#[derive(Clone)]
struct HubState {
  store: Arc<ThreadStore>,
}

pub fn serve(directory: PathBuf, port: u16) -> Result<(), HubError> {
  let runtime = match tokio::runtime::Builder::new_current_thread()
    .enable_io()
    .build()
  {
    Ok(runtime) => runtime,
    Err(_) => return Err(HubError::Runtime),
  };
  let result = runtime.block_on(serve_async(directory, port));

  return result;
}

async fn serve_async(directory: PathBuf, port: u16) -> Result<(), HubError> {
  let listener = match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
    Ok(listener) => listener,
    Err(_) => return Err(HubError::Bind),
  };
  let address = match listener.local_addr() {
    Ok(address) => address,
    Err(_) => return Err(HubError::Address),
  };
  let state = HubState {
    store: Arc::new(ThreadStore::new(directory)),
  };
  let application = Router::new()
    .route("/", get(home))
    .route("/threads", get(list_threads))
    .route("/threads/{id}", get(show_thread))
    .with_state(state);

  print_address(address);

  if axum::serve(listener, application).await.is_err() {
    return Err(HubError::Serve);
  }

  return Ok(());
}

fn print_address(address: SocketAddr) -> () {
  println!("Kermet Hub: http://{address}");

  return;
}

async fn home() -> Html<String> {
  let page = views::home_page().into_string();

  return Html(page);
}

async fn list_threads(State(state): State<HubState>) -> Result<Html<String>, HubResponseError> {
  let threads = match state.store.list() {
    Ok(threads) => threads,
    Err(_) => return Err(HubResponseError::ThreadStore),
  };
  let page = views::thread_list_page(&threads).into_string();

  return Ok(Html(page));
}

async fn show_thread(
  Path(id): Path<String>,
  State(state): State<HubState>,
) -> Result<Html<String>, HubResponseError> {
  let thread = match state.store.load(&id) {
    Ok(thread) => thread,
    Err(ThreadStoreError::NotFound) => return Err(HubResponseError::NotFound),
    Err(_) => return Err(HubResponseError::ThreadStore),
  };
  let page = views::thread_detail_page(&thread).into_string();

  return Ok(Html(page));
}

enum HubResponseError {
  NotFound,
  ThreadStore,
}

impl IntoResponse for HubResponseError {
  fn into_response(self) -> Response {
    let (status, title, message) = match self {
      Self::NotFound => (
        StatusCode::NOT_FOUND,
        "Thread not found",
        "The requested thread does not exist.",
      ),
      Self::ThreadStore => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Thread store error",
        "Kermet could not read the thread store.",
      ),
    };
    let page = views::error_page(title, message).into_string();
    let response = (status, Html(page)).into_response();

    return response;
  }
}
