use super::{SandboxError, SandboxOutput, SandboxPolicy, SandboxRequest};

pub trait SandboxRunner {
  fn execute(
    &self,
    policy: &SandboxPolicy,
    request: SandboxRequest,
  ) -> Result<SandboxOutput, SandboxError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxPlatform {
  MacOs,
  Linux,
  Windows,
  Unsupported,
}

impl SandboxPlatform {
  pub fn current() -> Self {
    #[cfg(target_os = "macos")]
    return Self::MacOs;

    #[cfg(target_os = "linux")]
    return Self::Linux;

    #[cfg(target_os = "windows")]
    return Self::Windows;

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Self::Unsupported;
  }
}

pub struct UnavailableSandbox {
  platform: SandboxPlatform,
}

impl UnavailableSandbox {
  pub fn new() -> Self {
    return Self {
      platform: SandboxPlatform::current(),
    };
  }
}

impl Default for UnavailableSandbox {
  fn default() -> Self {
    return Self::new();
  }
}

impl SandboxRunner for UnavailableSandbox {
  fn execute(
    &self,
    _policy: &SandboxPolicy,
    _request: SandboxRequest,
  ) -> Result<SandboxOutput, SandboxError> {
    return Err(SandboxError::Unavailable(self.platform));
  }
}
