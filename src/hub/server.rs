use axum::{
  Router,
  extract::{Request, State},
  http::StatusCode,
  middleware::{self, Next},
  response::{IntoResponse, Response},
  routing::get,
};
use std::{net::SocketAddr, path::PathBuf};

use super::{
  controllers::{HubState, bootstrap, home, list_threads, show_thread},
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
  let state = HubState::new(directory);
  print_address(address, state.access());
  let protected_routes = Router::new()
    .route("/", get(home))
    .route("/threads", get(list_threads))
    .route("/threads/{id}", get(show_thread))
    .route_layer(middleware::from_fn_with_state(
      state.clone(),
      require_read_access,
    ));
  let application = Router::new()
    .route("/bootstrap", get(bootstrap))
    .merge(protected_routes)
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
    AccessDecision::MutationProtectionRequired => return StatusCode::FORBIDDEN.into_response(),
  }

  let response = next.run(request).await;

  return response;
}
