#[cfg(not(target_os = "macos"))]
use super::super::UnavailableSandbox;
use super::super::{
  SandboxError, SandboxOutput, SandboxPlatform, SandboxPolicy, SandboxRequest, SandboxRunner,
};
#[cfg(target_os = "macos")]
use super::MacOsSandbox;

pub struct Sandbox {
  implementation: EnvironmentImplementation,
}

enum EnvironmentImplementation {
  #[cfg(target_os = "macos")]
  MacOs(MacOsSandbox),
  #[cfg(not(target_os = "macos"))]
  Unavailable(UnavailableSandbox),
}

impl Sandbox {
  pub fn current() -> Self {
    #[cfg(target_os = "macos")]
    let implementation = EnvironmentImplementation::MacOs(MacOsSandbox::new());

    #[cfg(not(target_os = "macos"))]
    let implementation = EnvironmentImplementation::Unavailable(UnavailableSandbox::new());

    return Self { implementation };
  }

  pub fn platform(&self) -> SandboxPlatform {
    let platform = match self.implementation {
      #[cfg(target_os = "macos")]
      EnvironmentImplementation::MacOs(_) => SandboxPlatform::MacOs,
      #[cfg(target_os = "linux")]
      EnvironmentImplementation::Unavailable(_) => SandboxPlatform::Linux,
      #[cfg(target_os = "windows")]
      EnvironmentImplementation::Unavailable(_) => SandboxPlatform::Windows,
      #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
      EnvironmentImplementation::Unavailable(_) => SandboxPlatform::Unsupported,
    };

    return platform;
  }
}

impl Default for Sandbox {
  fn default() -> Self {
    return Self::current();
  }
}

impl SandboxRunner for Sandbox {
  fn execute(
    &self,
    policy: &SandboxPolicy,
    request: SandboxRequest,
  ) -> Result<SandboxOutput, SandboxError> {
    let output = match self.implementation {
      #[cfg(target_os = "macos")]
      EnvironmentImplementation::MacOs(ref sandbox) => sandbox.execute(policy, request),
      #[cfg(not(target_os = "macos"))]
      EnvironmentImplementation::Unavailable(ref sandbox) => sandbox.execute(policy, request),
    };

    return output;
  }
}
