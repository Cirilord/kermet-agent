use std::{
  io::{Read, Write},
  net::{TcpListener, TcpStream},
  thread,
  time::{Duration, Instant},
};

use oauth2::CsrfToken;
use url::Url;

use super::error::CodexAuthError;

const CALLBACK_PATH: &str = "/auth/callback";
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(300);
const MAX_REQUEST_BYTES: usize = 16_384;

pub(super) struct Callback {
  pub(super) client_id: Option<String>,
  pub(super) code: String,
}

pub(super) struct CallbackServer {
  listener: TcpListener,
  redirect_uri: String,
}

impl CallbackServer {
  pub(super) fn start() -> Result<CallbackServer, CodexAuthError> {
    let listener = match TcpListener::bind("127.0.0.1:0") {
      Ok(listener) => listener,
      Err(_) => return Err(CodexAuthError::Callback),
    };

    if listener.set_nonblocking(true).is_err() {
      return Err(CodexAuthError::Callback);
    }
    let address = match listener.local_addr() {
      Ok(address) => address,
      Err(_) => return Err(CodexAuthError::Callback),
    };
    let port = address.port();
    let redirect_uri = format!("http://127.0.0.1:{port}{CALLBACK_PATH}");
    let server = CallbackServer {
      listener,
      redirect_uri,
    };

    return Ok(server);
  }

  pub(super) fn redirect_uri(&self) -> &str {
    return &self.redirect_uri;
  }

  pub(super) fn wait_for_callback(&self, state: &CsrfToken) -> Result<Callback, CodexAuthError> {
    let deadline = Instant::now() + CALLBACK_TIMEOUT;

    while Instant::now() < deadline {
      let mut stream = match self.listener.accept() {
        Ok((stream, _)) => stream,
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
          thread::sleep(Duration::from_millis(50));

          continue;
        }
        Err(_) => return Err(CodexAuthError::Callback),
      };
      let callback = parse_callback(&mut stream, state);

      match callback {
        Ok(callback) => {
          write_response(
            &mut stream,
            "200 OK",
            "Codex sign-in completed. You can close this window.",
          );

          return Ok(callback);
        }
        Err(CodexAuthError::AuthorizationDenied) => {
          write_response(
            &mut stream,
            "400 Bad Request",
            "Codex sign-in was denied. You can close this window.",
          );

          return Err(CodexAuthError::AuthorizationDenied);
        }
        Err(_) => {
          write_response(
            &mut stream,
            "400 Bad Request",
            "Codex sign-in could not be completed.",
          );
        }
      }
    }

    return Err(CodexAuthError::CallbackTimeout);
  }
}

fn parse_callback(
  stream: &mut TcpStream,
  expected_state: &CsrfToken,
) -> Result<Callback, CodexAuthError> {
  if stream
    .set_read_timeout(Some(Duration::from_secs(5)))
    .is_err()
  {
    return Err(CodexAuthError::Callback);
  }
  let request = match read_request(stream) {
    Ok(request) => request,
    Err(error) => return Err(error),
  };
  let target = match request_target(&request) {
    Ok(target) => target,
    Err(error) => return Err(error),
  };

  return callback_from_target(target, expected_state);
}

pub(super) fn callback_from_target(
  target: &str,
  expected_state: &CsrfToken,
) -> Result<Callback, CodexAuthError> {
  let url = match Url::parse(&format!("http://127.0.0.1{target}")) {
    Ok(url) => url,
    Err(_) => return Err(CodexAuthError::Callback),
  };

  if url.path() != CALLBACK_PATH {
    return Err(CodexAuthError::Callback);
  }

  let parameters = url.query_pairs().collect::<Vec<_>>();
  let state = match parameter(&parameters, "state") {
    Ok(state) => state,
    Err(error) => return Err(error),
  };

  let returned_state = CsrfToken::new(state.to_owned());

  if &returned_state != expected_state {
    return Err(CodexAuthError::Callback);
  }
  if parameter_optional(&parameters, "error").is_some() {
    return Err(CodexAuthError::AuthorizationDenied);
  }

  let callback = Callback {
    client_id: parameter_optional(&parameters, "client_id").map(str::to_owned),
    code: match parameter(&parameters, "code") {
      Ok(code) => code.to_owned(),
      Err(error) => return Err(error),
    },
  };

  return Ok(callback);
}

fn parameter<'a>(
  parameters: &'a [(std::borrow::Cow<'a, str>, std::borrow::Cow<'a, str>)],
  name: &str,
) -> Result<&'a str, CodexAuthError> {
  let value = match parameter_optional(parameters, name) {
    Some(value) => value,
    None => return Err(CodexAuthError::Callback),
  };

  return Ok(value);
}

fn parameter_optional<'a>(
  parameters: &'a [(std::borrow::Cow<'a, str>, std::borrow::Cow<'a, str>)],
  name: &str,
) -> Option<&'a str> {
  for parameter in parameters {
    if parameter.0.as_ref() == name {
      return Some(parameter.1.as_ref());
    }
  }

  return None;
}

fn read_request(stream: &mut TcpStream) -> Result<String, CodexAuthError> {
  let mut request = Vec::new();
  let mut buffer = [0_u8; 1024];

  loop {
    let read = match stream.read(&mut buffer) {
      Ok(read) => read,
      Err(_) => return Err(CodexAuthError::Callback),
    };

    if read == 0 {
      return Err(CodexAuthError::Callback);
    }
    request.extend_from_slice(&buffer[..read]);

    if request.len() > MAX_REQUEST_BYTES {
      return Err(CodexAuthError::Callback);
    }
    let mut has_headers = false;

    for window in request.windows(4) {
      if window == b"\r\n\r\n" {
        has_headers = true;

        break;
      }
    }
    if has_headers {
      break;
    }
  }

  let request = match String::from_utf8(request) {
    Ok(request) => request,
    Err(_) => return Err(CodexAuthError::Callback),
  };

  return Ok(request);
}

fn request_target(request: &str) -> Result<&str, CodexAuthError> {
  let request_line = match request.lines().next() {
    Some(request_line) => request_line,
    None => return Err(CodexAuthError::Callback),
  };
  let mut parts = request_line.split_whitespace();
  let method = match parts.next() {
    Some(method) => method,
    None => return Err(CodexAuthError::Callback),
  };
  let target = match parts.next() {
    Some(target) => target,
    None => return Err(CodexAuthError::Callback),
  };
  let version = match parts.next() {
    Some(version) => version,
    None => return Err(CodexAuthError::Callback),
  };

  if method != "GET" || version != "HTTP/1.1" || parts.next().is_some() {
    return Err(CodexAuthError::Callback);
  }

  return Ok(target);
}

fn write_response(stream: &mut TcpStream, status: &str, message: &str) -> () {
  let response = format!(
    "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{message}",
    message.len()
  );
  if stream.write_all(response.as_bytes()).is_err() {
    return;
  }
  if stream.flush().is_err() {
    return;
  }

  return;
}
