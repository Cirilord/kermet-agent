use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::sse::{Event, KeepAlive, Sse},
};
use std::convert::Infallible;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use super::super::controllers::HubState;

pub(in super::super) async fn thread_events(
  Path(id): Path<String>,
  State(state): State<HubState>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, StatusCode> {
  if state.store().load(&id).is_err() {
    return Err(StatusCode::NOT_FOUND);
  }

  let receiver = state.subscribe(&id);
  let event = Event::default().event("connected").data(id);
  let stream =
    tokio_stream::once(Ok(event)).chain(BroadcastStream::new(receiver).filter_map(|event| {
      let event = match event {
        Ok(event) => event,
        Err(_) => return None,
      };
      let event = Event::default().event(event.name).data(event.data);
      return Some(Ok(event));
    }));
  let response = Sse::new(stream).keep_alive(KeepAlive::default());

  return Ok(response);
}
