use axum::{
  extract::Request,
  http::{StatusCode, header},
  middleware::Next,
  response::{IntoResponse, Response},
};

const MAX_REQUEST_TARGET_BYTES: usize = 2 * 1024;
const MAX_REQUEST_HEADERS: usize = 32;
const MAX_REQUEST_HEADER_BYTES: usize = 8 * 1024;

pub(in super::super) async fn enforce_request_limits(request: Request, next: Next) -> Response {
  let violation = request_limit_violation(&request);

  if let Some(status) = violation {
    return status.into_response();
  }

  let response = next.run(request).await;

  return response;
}

pub(super) fn request_limit_violation(request: &Request) -> Option<StatusCode> {
  let request_target_size = match request.uri().path_and_query() {
    Some(path_and_query) => path_and_query.as_str().len(),
    None => 0,
  };

  if request_target_size > MAX_REQUEST_TARGET_BYTES {
    return Some(StatusCode::URI_TOO_LONG);
  }

  let headers = request.headers();

  if headers.len() > MAX_REQUEST_HEADERS {
    return Some(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE);
  }

  let mut header_size = 0_usize;

  for (name, value) in headers {
    let entry_size = match name.as_str().len().checked_add(value.as_bytes().len()) {
      Some(entry_size) => entry_size,
      None => return Some(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE),
    };
    header_size = match header_size.checked_add(entry_size) {
      Some(header_size) => header_size,
      None => return Some(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE),
    };
  }

  if header_size > MAX_REQUEST_HEADER_BYTES {
    return Some(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE);
  }

  if headers.contains_key(header::TRANSFER_ENCODING) {
    return Some(StatusCode::PAYLOAD_TOO_LARGE);
  }

  let content_length = match headers.get(header::CONTENT_LENGTH) {
    Some(content_length) => content_length,
    None => return None,
  };
  let content_length = match content_length.to_str() {
    Ok(content_length) => content_length,
    Err(_) => return Some(StatusCode::BAD_REQUEST),
  };
  let content_length = match content_length.parse::<usize>() {
    Ok(content_length) => content_length,
    Err(_) => return Some(StatusCode::BAD_REQUEST),
  };

  let path = request.uri().path();
  let accepts_body = request.method() == axum::http::Method::POST
    && (path == "/api/threads"
      || (path.starts_with("/api/threads/") && path.ends_with("/messages")));

  if content_length > 4096 || (content_length > 0 && !accepts_body) {
    return Some(StatusCode::PAYLOAD_TOO_LARGE);
  }

  return None;
}
