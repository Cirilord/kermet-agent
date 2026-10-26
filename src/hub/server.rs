use axum::{
  Router,
  extract::{Request, State},
  http::StatusCode,
  middleware::{self, Next},
  response::{IntoResponse, Response},
  routing::{get, post},
};
use std::{net::SocketAddr, path::PathBuf};

use super::{
  api::{create_message, create_thread, open_workspace_picker, start_initial_turn, thread_events},
  controllers::{HubState, bootstrap, home, list_threads, new_thread, show_thread},
  security::{
    auth::{AccessDecision, HubAccess, RouteAccess},
    request_limits::enforce_request_limits,
    response_headers::add_response_headers,
  },
};

#[derive(Debug)]
pub enum HubError {
  Runtime,
  Bind,
  Address,
  Serve,
}

pub fn serve(
  thread_directory: PathBuf,
  workspace_directory: PathBuf,
  port: u16,
) -> Result<(), HubError> {
  let runtime = match tokio::runtime::Builder::new_current_thread()
    .enable_io()
    .enable_time()
    .build()
  {
    Ok(runtime) => runtime,
    Err(_) => return Err(HubError::Runtime),
  };
  let result = runtime.block_on(serve_async(thread_directory, workspace_directory, port));

  return result;
}

async fn serve_async(
  thread_directory: PathBuf,
  workspace_directory: PathBuf,
  port: u16,
) -> Result<(), HubError> {
  let listener = match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
    Ok(listener) => listener,
    Err(_) => return Err(HubError::Bind),
  };
  let address = match listener.local_addr() {
    Ok(address) => address,
    Err(_) => return Err(HubError::Address),
  };
  let origin = format!("http://{address}");
  let state = HubState::new(thread_directory, workspace_directory, origin);
  print_address(address, state.access());
  let protected_routes = Router::new()
    .route("/", get(home))
    .route("/threads", get(list_threads))
    .route("/threads/new", get(new_thread))
    .route("/threads/{id}", get(show_thread))
    .route("/api/threads/{id}/events", get(thread_events))
    .route_layer(middleware::from_fn_with_state(
      state.clone(),
      require_read_access,
    ));
  let mutation_routes = Router::new()
    .route("/api/workspaces/picker", post(open_workspace_picker))
    .route("/api/threads", post(create_thread))
    .route("/api/threads/{id}/initial-turn", post(start_initial_turn))
    .route("/api/threads/{id}/messages", post(create_message))
    .route_layer(middleware::from_fn_with_state(
      state.clone(),
      require_mutation_access,
    ));
  let application = Router::new()
    .route("/bootstrap", get(bootstrap))
    .merge(protected_routes)
    .merge(mutation_routes)
    .layer(middleware::from_fn(enforce_request_limits))
    .with_state(state)
    .layer(middleware::from_fn(add_response_headers));

  if axum::serve(listener, application).await.is_err() {
    return Err(HubError::Serve);
  }

  return Ok(());
}

fn print_address(address: SocketAddr, access: &HubAccess) -> () {
  println!(
    "Kermet Hub: http://{address}/bootstrap?token={}",
    access.bootstrap_token()
  );

  return;
}

async fn require_read_access(
  State(state): State<HubState>,
  request: Request,
  next: Next,
) -> Response {
  let decision = state
    .access()
    .authorize(request.headers(), RouteAccess::Read);

  match decision {
    AccessDecision::Allowed => (),
    AccessDecision::Unauthenticated => return StatusCode::UNAUTHORIZED.into_response(),
    AccessDecision::MutationProtectionRequired
    | AccessDecision::InvalidOrigin
    | AccessDecision::InvalidCsrfToken => return StatusCode::FORBIDDEN.into_response(),
  }

  let response = next.run(request).await;

  return response;
}

async fn require_mutation_access(
  State(state): State<HubState>,
  request: Request,
  next: Next,
) -> Response {
  let decision = state
    .access()
    .authorize_mutation(request.headers(), state.origin());

  match decision {
    AccessDecision::Allowed => (),
    AccessDecision::Unauthenticated => return StatusCode::UNAUTHORIZED.into_response(),
    AccessDecision::MutationProtectionRequired
    | AccessDecision::InvalidOrigin
    | AccessDecision::InvalidCsrfToken => return StatusCode::FORBIDDEN.into_response(),
  }

  let response = next.run(request).await;

  return response;
}
