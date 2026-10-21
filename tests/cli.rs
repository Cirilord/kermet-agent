use std::{
  fs,
  path::PathBuf,
  process::Command,
  sync::atomic::{AtomicU64, Ordering},
  time::{SystemTime, UNIX_EPOCH},
};

static WORKSPACE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn create_workspace() -> PathBuf {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("the system clock should be after the Unix epoch")
    .as_nanos();
  let sequence = WORKSPACE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
  let workspace = std::env::temp_dir().join(format!(
    "kermet-cli-test-{}-{timestamp}-{sequence}",
    std::process::id(),
  ));

  fs::create_dir(&workspace).expect("the temporary workspace should be created");
  let workspace =
    fs::canonicalize(workspace).expect("the temporary workspace path should be canonicalized");

  return workspace;
}

#[test]
fn binary_prints_help() -> () {
  let output = Command::new(env!("CARGO_BIN_EXE_kermet"))
    .arg("--help")
    .output()
    .expect("the Kermet binary should execute");

  assert!(output.status.success());

  let stdout = String::from_utf8(output.stdout).expect("help output should be valid UTF-8");

  assert!(stdout.contains("A local coding agent"));
  assert!(stdout.contains("doctor"));
  assert!(stdout.contains("run"));
  assert!(stdout.contains("login"));
  assert!(stdout.contains("logout"));

  return;
}

#[test]
fn binary_prints_help_without_a_command() -> () {
  let output = Command::new(env!("CARGO_BIN_EXE_kermet"))
    .output()
    .expect("the Kermet binary should execute");

  assert!(output.status.success());

  let stdout = String::from_utf8(output.stdout).expect("help output should be valid UTF-8");

  assert!(stdout.contains("A local coding agent"));
  assert!(stdout.contains("doctor"));
  assert!(stdout.contains("run"));
  assert!(stdout.contains("login"));
  assert!(stdout.contains("logout"));

  return;
}

#[test]
fn binary_reports_its_version() -> () {
  let output = Command::new(env!("CARGO_BIN_EXE_kermet"))
    .arg("--version")
    .output()
    .expect("the Kermet binary should execute");

  assert!(output.status.success());

  let stdout = String::from_utf8(output.stdout).expect("version output should be valid UTF-8");

  assert_eq!(stdout, format!("kermet {}\n", env!("CARGO_PKG_VERSION")));

  return;
}

#[test]
fn doctor_inspects_the_requested_workspace() -> () {
  let workspace = create_workspace();
  let output = Command::new(env!("CARGO_BIN_EXE_kermet"))
    .args(["doctor", "--workspace"])
    .arg(&workspace)
    .output()
    .expect("the Kermet binary should execute");

  fs::remove_dir_all(&workspace).expect("the temporary workspace should be removed");

  assert!(output.status.success());

  let stdout = String::from_utf8(output.stdout).expect("doctor output should be valid UTF-8");

  assert!(stdout.contains(&format!("Workspace: {}", workspace.display())));
  assert!(stdout.contains("Git metadata: not detected"));
  assert!(stdout.contains("Model provider: not configured"));

  return;
}

#[test]
fn run_fails_when_no_model_provider_is_configured() -> () {
  let workspace = create_workspace();
  let output = Command::new(env!("CARGO_BIN_EXE_kermet"))
    .args(["run", "inspect the project", "--workspace"])
    .arg(&workspace)
    .output()
    .expect("the Kermet binary should execute");

  fs::remove_dir_all(&workspace).expect("the temporary workspace should be removed");

  assert_eq!(output.status.code(), Some(78_i32));

  let stderr = String::from_utf8(output.stderr).expect("run error output should be valid UTF-8");

  assert!(stderr.contains("error: no model provider is configured"));
  assert!(stderr.contains("task: inspect the project"));
  assert!(stderr.contains(&format!("workspace: {}", workspace.display())));
  assert!(stderr.contains("permission mode: read-only"));

  return;
}

#[test]
fn run_rejects_an_empty_task() -> () {
  let workspace = create_workspace();
  let output = Command::new(env!("CARGO_BIN_EXE_kermet"))
    .args(["run", "  ", "--workspace"])
    .arg(&workspace)
    .output()
    .expect("the Kermet binary should execute");

  fs::remove_dir_all(&workspace).expect("the temporary workspace should be removed");

  assert_eq!(output.status.code(), Some(2_i32));

  let stderr = String::from_utf8(output.stderr).expect("run error output should be valid UTF-8");

  assert!(stderr.contains("error: task must not be empty"));

  return;
}
