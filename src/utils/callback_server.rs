use std::{
  io::{Read, Write},
  net::TcpListener,
};

pub struct CallbackServer {
  listener: TcpListener,
}

impl CallbackServer {
  pub fn bind() -> Result<Self, CallbackServerError> {
    let listener = match TcpListener::bind("127.0.0.1:1455") {
      Ok(listener) => listener,
      Err(error) => return Err(CallbackServerError::Bind(error)),
    };
    return Ok(Self { listener });
  }

  pub fn receive(&self) -> Result<String, CallbackServerError> {
    let (mut stream, _) = match self.listener.accept() {
      Ok(connection) => connection,
      Err(error) => return Err(CallbackServerError::Accept(error)),
    };
    let mut buffer = [0_u8; 4096];
    let bytes_read = match stream.read(&mut buffer) {
      Ok(bytes_read) => bytes_read,
      Err(error) => return Err(CallbackServerError::Read(error)),
    };
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let target = match request.lines().next().and_then(|line| {
      return line.split_whitespace().nth(1);
    }) {
      Some(target) => target.to_owned(),
      None => return Err(CallbackServerError::InvalidRequest),
    };
    if let Err(error) =
      stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
    {
      return Err(CallbackServerError::Write(error));
    }
    return Ok(target);
  }
}

#[derive(Debug)]
pub enum CallbackServerError {
  Bind(std::io::Error),
  Accept(std::io::Error),
  Read(std::io::Error),
  Write(std::io::Error),
  InvalidRequest,
}
