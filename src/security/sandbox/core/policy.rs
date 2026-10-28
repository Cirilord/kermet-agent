use std::{
  path::{Path, PathBuf},
  time::Duration,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_MAX_OUTPUT_BYTES: usize = 32 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxMode {
  ReadOnly,
  WorkspaceWrite,
  DangerFullAccess,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkAccess {
  Denied,
  Allowed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
  timeout: Duration,
  max_output_bytes: usize,
}

impl ResourceLimits {
  pub fn new(timeout: Duration, max_output_bytes: usize) -> Result<Self, SandboxPolicyError> {
    if timeout.is_zero() || max_output_bytes == 0 {
      return Err(SandboxPolicyError::InvalidResourceLimits);
    }

    return Ok(Self {
      timeout,
      max_output_bytes,
    });
  }

  pub fn timeout(&self) -> Duration {
    return self.timeout;
  }

  pub fn max_output_bytes(&self) -> usize {
    return self.max_output_bytes;
  }
}

impl Default for ResourceLimits {
  fn default() -> Self {
    return Self {
      timeout: DEFAULT_TIMEOUT,
      max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
    };
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxPolicy {
  mode: SandboxMode,
  workspace: PathBuf,
  network_access: NetworkAccess,
  resource_limits: ResourceLimits,
}

impl SandboxPolicy {
  pub fn new(
    mode: SandboxMode,
    workspace: PathBuf,
    network_access: NetworkAccess,
    resource_limits: ResourceLimits,
  ) -> Result<Self, SandboxPolicyError> {
    if workspace.as_os_str().is_empty() {
      return Err(SandboxPolicyError::InvalidWorkspace);
    }

    return Ok(Self {
      mode,
      workspace,
      network_access,
      resource_limits,
    });
  }

  pub fn read_only(workspace: PathBuf) -> Result<Self, SandboxPolicyError> {
    let policy = Self::new(
      SandboxMode::ReadOnly,
      workspace,
      NetworkAccess::Denied,
      ResourceLimits::default(),
    );

    return policy;
  }

  pub fn workspace_write(workspace: PathBuf) -> Result<Self, SandboxPolicyError> {
    let policy = Self::new(
      SandboxMode::WorkspaceWrite,
      workspace,
      NetworkAccess::Denied,
      ResourceLimits::default(),
    );

    return policy;
  }

  pub fn danger_full_access(workspace: PathBuf) -> Result<Self, SandboxPolicyError> {
    let policy = Self::new(
      SandboxMode::DangerFullAccess,
      workspace,
      NetworkAccess::Allowed,
      ResourceLimits::default(),
    );

    return policy;
  }

  pub fn mode(&self) -> SandboxMode {
    return self.mode;
  }

  pub fn workspace(&self) -> &Path {
    return &self.workspace;
  }

  pub fn network_access(&self) -> NetworkAccess {
    return self.network_access;
  }

  pub fn resource_limits(&self) -> ResourceLimits {
    return self.resource_limits;
  }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SandboxPolicyError {
  InvalidWorkspace,
  InvalidResourceLimits,
}
