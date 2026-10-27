use super::{BashOutput, BashRequest};
use crate::{
  agent::TaskRequest,
  security::{Capability, PermissionDenied, PermissionManager},
};
use std::{
  io::Read,
  process::{Child, ChildStderr, ChildStdout, Command, Stdio},
  thread::{self, JoinHandle},
  time::{Duration, Instant},
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_STREAM_BYTES: usize = 32 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct Bash {
  permission_manager: PermissionManager,
  timeout: Duration,
}

impl Bash {
  pub fn new(permission_manager: PermissionManager) -> Self {
    return Self {
      permission_manager,
      timeout: DEFAULT_TIMEOUT,
    };
  }

  #[cfg(test)]
  pub(super) fn with_timeout(permission_manager: PermissionManager, timeout: Duration) -> Self {
    return Self {
      permission_manager,
      timeout,
    };
  }

  pub fn execute(&self, task: &TaskRequest, request: BashRequest) -> Result<BashOutput, BashError> {
    if let Err(error) = self
      .permission_manager
      .authorize(task.permission_mode(), Capability::ExecuteProcess)
    {
      return Err(BashError::PermissionDenied(error));
    }

    let path = std::env::var_os("PATH");
    let mut command = Command::new("bash");
    command
      .arg("-c")
      .arg(request.command())
      .current_dir(task.workspace().path())
      .env_clear()
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped());
    if let Some(path) = path {
      command.env("PATH", path);
    }
    let mut child = match command.spawn() {
      Ok(child) => child,
      Err(source) => return Err(BashError::Spawn(source)),
    };
    let stdout = match child.stdout.take() {
      Some(stdout) => stdout,
      None => return Err(BashError::MissingOutputStream),
    };
    let stderr = match child.stderr.take() {
      Some(stderr) => stderr,
      None => return Err(BashError::MissingErrorStream),
    };
    let stdout_reader = capture_stdout(stdout);
    let stderr_reader = capture_stderr(stderr);
    let (exit_code, timed_out) = match wait_for_child(&mut child, self.timeout) {
      Ok(result) => result,
      Err(error) => return Err(error),
    };
    let stdout = match join_capture(stdout_reader) {
      Ok(output) => output,
      Err(error) => return Err(error),
    };
    let stderr = match join_capture(stderr_reader) {
      Ok(output) => output,
      Err(error) => return Err(error),
    };
    let output = BashOutput::new(
      exit_code,
      timed_out,
      stdout.contents,
      stderr.contents,
      stdout.truncated,
      stderr.truncated,
    );

    return Ok(output);
  }
}

#[derive(Debug)]
pub enum BashError {
  PermissionDenied(PermissionDenied),
  Spawn(std::io::Error),
  MissingOutputStream,
  MissingErrorStream,
  Wait(std::io::Error),
  Kill(std::io::Error),
  Capture(std::io::Error),
  CaptureThread,
}

struct CapturedStream {
  contents: Vec<u8>,
  truncated: bool,
}

fn capture_stdout(stdout: ChildStdout) -> JoinHandle<Result<CapturedStream, std::io::Error>> {
  let handle = thread::spawn(move || return capture_stream(stdout));

  return handle;
}

fn capture_stderr(stderr: ChildStderr) -> JoinHandle<Result<CapturedStream, std::io::Error>> {
  let handle = thread::spawn(move || return capture_stream(stderr));

  return handle;
}

fn capture_stream<R: Read>(mut stream: R) -> Result<CapturedStream, std::io::Error> {
  let mut contents = Vec::new();
  let mut truncated = false;
  let mut buffer = [0_u8; 4096];

  loop {
    let bytes_read = match stream.read(&mut buffer) {
      Ok(bytes_read) => bytes_read,
      Err(error) => return Err(error),
    };
    if bytes_read == 0 {
      break;
    }
    let remaining = MAX_STREAM_BYTES.saturating_sub(contents.len());
    let bytes_to_keep = remaining.min(bytes_read);
    contents.extend_from_slice(&buffer[..bytes_to_keep]);
    if bytes_to_keep < bytes_read {
      truncated = true;
    }
  }

  return Ok(CapturedStream {
    contents,
    truncated,
  });
}

fn wait_for_child(child: &mut Child, timeout: Duration) -> Result<(Option<i32>, bool), BashError> {
  let deadline = match Instant::now().checked_add(timeout) {
    Some(deadline) => deadline,
    None => {
      return Err(BashError::Wait(std::io::Error::other(
        "invalid process timeout",
      )));
    }
  };

  loop {
    let status = match child.try_wait() {
      Ok(status) => status,
      Err(source) => return Err(BashError::Wait(source)),
    };
    if let Some(status) = status {
      return Ok((status.code(), false));
    }
    if Instant::now() >= deadline {
      if let Err(source) = child.kill() {
        return Err(BashError::Kill(source));
      }
      let status = match child.wait() {
        Ok(status) => status,
        Err(source) => return Err(BashError::Wait(source)),
      };

      return Ok((status.code(), true));
    }
    thread::sleep(POLL_INTERVAL);
  }
}

fn join_capture(
  handle: JoinHandle<Result<CapturedStream, std::io::Error>>,
) -> Result<CapturedStream, BashError> {
  let result = match handle.join() {
    Ok(result) => result,
    Err(_) => return Err(BashError::CaptureThread),
  };
  let capture = match result {
    Ok(capture) => capture,
    Err(error) => return Err(BashError::Capture(error)),
  };

  return Ok(capture);
}
