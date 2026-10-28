#[derive(Debug, Eq, PartialEq)]
pub struct SandboxOutput {
  exit_code: Option<i32>,
  timed_out: bool,
  stdout: Vec<u8>,
  stderr: Vec<u8>,
  stdout_truncated: bool,
  stderr_truncated: bool,
}

impl SandboxOutput {
  pub fn new(
    exit_code: Option<i32>,
    timed_out: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdout_truncated: bool,
    stderr_truncated: bool,
  ) -> Self {
    return Self {
      exit_code,
      timed_out,
      stdout,
      stderr,
      stdout_truncated,
      stderr_truncated,
    };
  }

  pub fn exit_code(&self) -> Option<i32> {
    return self.exit_code;
  }

  pub fn timed_out(&self) -> bool {
    return self.timed_out;
  }

  pub fn stdout(&self) -> &[u8] {
    return &self.stdout;
  }

  pub fn stderr(&self) -> &[u8] {
    return &self.stderr;
  }

  pub fn stdout_truncated(&self) -> bool {
    return self.stdout_truncated;
  }

  pub fn stderr_truncated(&self) -> bool {
    return self.stderr_truncated;
  }
}
