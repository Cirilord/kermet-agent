use axum::{
  extract::{Query, State},
  http::{StatusCode, header},
  response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use super::{super::security::auth::SessionTransport, HubState};

#[derive(Deserialize)]
pub(in super::super) struct BootstrapQuery {
  pub(in super::super) token: String,
}

pub(in super::super) async fn bootstrap(
  State(state): State<HubState>,
  Query(query): Query<BootstrapQuery>,
) -> Response {
  if !state.access().consume_bootstrap_token(&query.token) {
    return StatusCode::UNAUTHORIZED.into_response();
  }

  let response = (
    [(
      header::SET_COOKIE,
      state.access().session_cookie(SessionTransport::Http),
    )],
    Redirect::to("/"),
  )
    .into_response();

  return response;
}
