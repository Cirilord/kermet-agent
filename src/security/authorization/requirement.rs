use super::super::sandbox::{NetworkAccess, SandboxMode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionRequirement {
  sandbox_mode: SandboxMode,
  network_access: NetworkAccess,
}

impl ExecutionRequirement {
  pub fn new(sandbox_mode: SandboxMode, network_access: NetworkAccess) -> Self {
    return Self {
      sandbox_mode,
      network_access,
    };
  }

  pub fn sandbox_mode(&self) -> SandboxMode {
    return self.sandbox_mode;
  }

  pub fn network_access(&self) -> NetworkAccess {
    return self.network_access;
  }
}
