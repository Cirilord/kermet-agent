use crate::agent::PermissionMode;
use crate::security::Capability;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default)]
pub struct PermissionManager;

impl PermissionManager {
  pub fn new() -> Self {
    return Self;
  }

  pub fn authorize(
    &self,
    permission_mode: PermissionMode,
    capability: Capability,
  ) -> Result<(), PermissionDenied> {
    let is_allowed = match permission_mode {
      PermissionMode::ReadOnly => {
        matches!(capability, Capability::ReadFile | Capability::SearchFiles)
      }
      PermissionMode::WorkspaceWrite => matches!(
        capability,
        Capability::ReadFile | Capability::SearchFiles | Capability::WriteFile
      ),
    };

    if !is_allowed {
      return Err(PermissionDenied {
        permission_mode,
        capability,
      });
    }

    return Ok(());
  }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PermissionDenied {
  permission_mode: PermissionMode,
  capability: Capability,
}

impl PermissionDenied {
  pub fn capability(&self) -> Capability {
    return self.capability;
  }

  pub fn permission_mode(&self) -> PermissionMode {
    return self.permission_mode;
  }
}

impl Display for PermissionDenied {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let result = write!(
      formatter,
      "capability '{}' is denied in '{}' mode",
      self.capability, self.permission_mode
    );

    return result;
  }
}

impl std::error::Error for PermissionDenied {}

#[cfg(test)]
mod tests {
  use super::PermissionManager;
  use crate::agent::PermissionMode;
  use crate::security::Capability;

  #[test]
  fn allows_read_capabilities_in_read_only_mode() -> () {
    let manager = PermissionManager::new();

    assert!(
      manager
        .authorize(PermissionMode::ReadOnly, Capability::ReadFile)
        .is_ok()
    );
    assert!(
      manager
        .authorize(PermissionMode::ReadOnly, Capability::SearchFiles)
        .is_ok()
    );

    return;
  }

  #[test]
  fn denies_write_and_process_capabilities_in_read_only_mode() -> () {
    let manager = PermissionManager::new();
    let write_error = manager
      .authorize(PermissionMode::ReadOnly, Capability::WriteFile)
      .expect_err("read-only mode should deny file writes");
    let process_error = manager
      .authorize(PermissionMode::ReadOnly, Capability::ExecuteProcess)
      .expect_err("read-only mode should deny process execution");

    assert_eq!(write_error.permission_mode(), PermissionMode::ReadOnly);
    assert_eq!(write_error.capability(), Capability::WriteFile);
    assert_eq!(process_error.permission_mode(), PermissionMode::ReadOnly);
    assert_eq!(process_error.capability(), Capability::ExecuteProcess);

    return;
  }

  #[test]
  fn allows_workspace_file_capabilities_in_workspace_write_mode() -> () {
    let manager = PermissionManager::new();

    assert!(
      manager
        .authorize(PermissionMode::WorkspaceWrite, Capability::ReadFile)
        .is_ok()
    );
    assert!(
      manager
        .authorize(PermissionMode::WorkspaceWrite, Capability::SearchFiles)
        .is_ok()
    );
    assert!(
      manager
        .authorize(PermissionMode::WorkspaceWrite, Capability::WriteFile)
        .is_ok()
    );

    return;
  }

  #[test]
  fn denies_process_execution_in_workspace_write_mode() -> () {
    let manager = PermissionManager::new();
    let error = manager
      .authorize(PermissionMode::WorkspaceWrite, Capability::ExecuteProcess)
      .expect_err("workspace-write mode should deny process execution");

    assert_eq!(error.permission_mode(), PermissionMode::WorkspaceWrite);
    assert_eq!(error.capability(), Capability::ExecuteProcess);

    return;
  }
}
