use super::super::{
  NetworkAccess, SandboxError, SandboxMode, SandboxOutput, SandboxPolicy, SandboxRequest,
  SandboxRunner,
};
use std::{
  io::Read,
  process::{Child, ChildStderr, ChildStdout, Command, Stdio},
  thread::{self, JoinHandle},
  time::Instant,
};

const SANDBOX_EXECUTABLE: &str = "/usr/bin/sandbox-exec";

pub struct MacOsSandbox;

impl MacOsSandbox {
  pub fn new() -> Self {
    return Self;
  }

  pub(in crate::security::sandbox) fn profile(
    policy: &SandboxPolicy,
  ) -> Result<String, SandboxError> {
    if policy.mode() == SandboxMode::DangerFullAccess {
      if policy.network_access() != NetworkAccess::Allowed {
        return Err(SandboxError::InvalidPolicy);
      }

      return Ok("(version 1) (allow default)".to_owned());
    }

    let workspace = match escaped_workspace(policy) {
      Ok(workspace) => workspace,
      Err(error) => return Err(error),
    };
    let mut profile = String::from(
      r#"(version 1)
      (deny default)
      (import "system.sb")
      (allow process-exec)
      (allow process-fork)
      (allow file-read* (subpath "/bin"))
      (allow file-read* (subpath "/sbin"))
      (allow file-read* (subpath "/usr/bin"))
      (allow file-read* (subpath "/usr/lib"))
      (allow file-read* (subpath "/System/Library"))
      (allow file-read* (subpath "/Library/Developer"))
      (allow file-read* (subpath "/Applications/Xcode.app"))
      (allow file-read* (subpath "/opt/homebrew"))
      (allow file-read* (subpath "/dev"))
      "#,
    );
    profile.push_str(&format!("(allow file-read* (subpath \"{workspace}\"))\n"));
    if policy.mode() == SandboxMode::WorkspaceWrite {
      profile.push_str(&format!("(allow file-write* (subpath \"{workspace}\"))\n"));
    }
    if policy.network_access() == NetworkAccess::Allowed {
      profile.push_str("(allow network*)\n");
    }

    return Ok(profile);
  }
}

impl Default for MacOsSandbox {
  fn default() -> Self {
    return Self::new();
  }
}

impl SandboxRunner for MacOsSandbox {
  fn execute(
    &self,
    policy: &SandboxPolicy,
    request: SandboxRequest,
  ) -> Result<SandboxOutput, SandboxError> {
    let profile = match Self::profile(policy) {
      Ok(profile) => profile,
      Err(error) => return Err(error),
    };
    let mut command = Command::new(SANDBOX_EXECUTABLE);
    command
      .arg("-p")
      .arg(profile)
      .arg(request.program())
      .args(request.arguments())
      .current_dir(policy.workspace())
      .env_clear()
      .envs(request.environment())
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped());
    let mut child = match command.spawn() {
      Ok(child) => child,
      Err(source) => return Err(SandboxError::Spawn(source)),
    };
    let stdout = match child.stdout.take() {
      Some(stdout) => stdout,
      None => return Err(SandboxError::MissingOutputStream),
    };
    let stderr = match child.stderr.take() {
      Some(stderr) => stderr,
      None => return Err(SandboxError::MissingErrorStream),
    };
    let max_output_bytes = policy.resource_limits().max_output_bytes();
    let stdout_reader = capture_stdout(stdout, max_output_bytes);
    let stderr_reader = capture_stderr(stderr, max_output_bytes);
    let (exit_code, timed_out) = match wait_for_child(&mut child, policy) {
      Ok(result) => result,
      Err(error) => return Err(error),
    };
    let stdout = match join_capture(stdout_reader) {
      Ok(capture) => capture,
      Err(error) => return Err(error),
    };
    let stderr = match join_capture(stderr_reader) {
      Ok(capture) => capture,
      Err(error) => return Err(error),
    };

    return Ok(SandboxOutput::new(
      exit_code,
      timed_out,
      stdout.contents,
      stderr.contents,
      stdout.truncated,
      stderr.truncated,
    ));
  }
}

struct CapturedStream {
  contents: Vec<u8>,
  truncated: bool,
}

fn escaped_workspace(policy: &SandboxPolicy) -> Result<String, SandboxError> {
  let workspace = match policy.workspace().to_str() {
    Some(workspace) => workspace,
    None => return Err(SandboxError::InvalidPolicy),
  };
  if workspace.chars().any(char::is_control) {
    return Err(SandboxError::InvalidPolicy);
  }

  let escaped = workspace.replace('\\', "\\\\").replace('"', "\\\"");

  return Ok(escaped);
}

fn capture_stdout(
  stdout: ChildStdout,
  max_output_bytes: usize,
) -> JoinHandle<Result<CapturedStream, std::io::Error>> {
  let handle = thread::spawn(move || return capture_stream(stdout, max_output_bytes));

  return handle;
}

fn capture_stderr(
  stderr: ChildStderr,
  max_output_bytes: usize,
) -> JoinHandle<Result<CapturedStream, std::io::Error>> {
  let handle = thread::spawn(move || return capture_stream(stderr, max_output_bytes));

  return handle;
}

fn capture_stream<R: Read>(
  mut stream: R,
  max_output_bytes: usize,
) -> Result<CapturedStream, std::io::Error> {
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
    let remaining = max_output_bytes.saturating_sub(contents.len());
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

fn wait_for_child(
  child: &mut Child,
  policy: &SandboxPolicy,
) -> Result<(Option<i32>, bool), SandboxError> {
  let deadline = match Instant::now().checked_add(policy.resource_limits().timeout()) {
    Some(deadline) => deadline,
    None => return Err(SandboxError::InvalidPolicy),
  };

  loop {
    let status = match child.try_wait() {
      Ok(status) => status,
      Err(source) => return Err(SandboxError::Wait(source)),
    };
    if let Some(status) = status {
      return Ok((status.code(), false));
    }
    if Instant::now() >= deadline {
      if let Err(source) = child.kill() {
        return Err(SandboxError::Kill(source));
      }
      let status = match child.wait() {
        Ok(status) => status,
        Err(source) => return Err(SandboxError::Wait(source)),
      };

      return Ok((status.code(), true));
    }
    std::thread::sleep(std::time::Duration::from_millis(10));
  }
}

fn join_capture(
  handle: JoinHandle<Result<CapturedStream, std::io::Error>>,
) -> Result<CapturedStream, SandboxError> {
  let result = match handle.join() {
    Ok(result) => result,
    Err(_) => return Err(SandboxError::CaptureThread),
  };
  let capture = match result {
    Ok(capture) => capture,
    Err(error) => return Err(SandboxError::Capture(error)),
  };

  return Ok(capture);
}
