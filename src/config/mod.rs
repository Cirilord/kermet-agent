use crate::{
  models::{ModelSelection, ModelSelectionError, ReasoningEffort},
  security::{
    ApprovalPolicy, ExecutionPolicy, ExecutionPolicyError,
    sandbox::{NetworkAccess, SandboxMode},
  },
};
use serde::Deserialize;
use std::{
  collections::HashMap,
  fs::{self, File, OpenOptions},
  io::Write,
  path::{Path, PathBuf},
};

pub struct ModelOverrides {
  pub provider: Option<String>,
  pub model: Option<String>,
  pub reasoning_effort: Option<ReasoningEffort>,
}

impl ModelOverrides {
  pub fn none() -> Self {
    return Self {
      provider: None,
      model: None,
      reasoning_effort: None,
    };
  }
}

pub struct ExecutionOverrides {
  pub sandbox_mode: Option<SandboxMode>,
  pub network_access: Option<NetworkAccess>,
  pub approval_policy: Option<ApprovalPolicy>,
}

impl ExecutionOverrides {
  pub fn none() -> Self {
    return Self {
      sandbox_mode: None,
      network_access: None,
      approval_policy: None,
    };
  }
}

pub fn default_path(home: &Path) -> PathBuf {
  return home.join(".kermet").join("config.toml");
}

pub fn user_path() -> Result<PathBuf, ConfigError> {
  let home = match std::env::var_os("HOME") {
    Some(home) => PathBuf::from(home),
    None => return Err(ConfigError::HomeUnavailable),
  };
  let path = default_path(&home);

  return Ok(path);
}

pub fn resolve_model_selection(
  path: &Path,
  profile: &str,
  overrides: ModelOverrides,
) -> Result<ModelSelection, ConfigError> {
  let configuration = match Configuration::load(path) {
    Ok(configuration) => configuration,
    Err(error) => return Err(error),
  };
  let profile_configuration = configuration.profiles.get(profile);
  let provider = match overrides.provider {
    Some(provider) => Some(provider),
    None => profile_configuration.and_then(|configuration| return configuration.provider.clone()),
  };
  let model = match overrides.model {
    Some(model) => Some(model),
    None => profile_configuration.and_then(|configuration| return configuration.model.clone()),
  };
  let reasoning_effort = match overrides.reasoning_effort {
    Some(reasoning_effort) => Some(reasoning_effort),
    None => profile_configuration.and_then(|configuration| return configuration.reasoning_effort),
  };
  let provider = match provider {
    Some(provider) => provider,
    None => return Err(ConfigError::MissingProvider),
  };
  let model = match model {
    Some(model) => model,
    None => return Err(ConfigError::MissingModel),
  };
  let reasoning_effort = match reasoning_effort {
    Some(reasoning_effort) => reasoning_effort,
    None => return Err(ConfigError::MissingReasoningEffort),
  };
  let selection = match ModelSelection::new(provider, model, reasoning_effort) {
    Ok(selection) => selection,
    Err(error) => return Err(ConfigError::InvalidSelection(error)),
  };

  return Ok(selection);
}

pub fn resolve_execution_policy(
  path: &Path,
  overrides: ExecutionOverrides,
) -> Result<ExecutionPolicy, ConfigError> {
  let configuration = match Configuration::load(path) {
    Ok(configuration) => configuration,
    Err(error) => return Err(error),
  };
  let sandbox_mode = match overrides.sandbox_mode {
    Some(mode) => Some(mode),
    None => configuration.execution.sandbox_mode(),
  };
  let network_access = match overrides.network_access {
    Some(access) => Some(access),
    None => configuration.execution.network_access(),
  };
  let approval_policy = match overrides.approval_policy {
    Some(policy) => Some(policy),
    None => configuration.execution.approval_policy(),
  };
  let sandbox_mode = match sandbox_mode {
    Some(mode) => mode,
    None => return Err(ConfigError::MissingSandboxMode),
  };
  let network_access = match network_access {
    Some(access) => access,
    None => return Err(ConfigError::MissingNetworkAccess),
  };
  let approval_policy = match approval_policy {
    Some(policy) => policy,
    None => return Err(ConfigError::MissingApprovalPolicy),
  };
  let policy = match ExecutionPolicy::new(sandbox_mode, network_access, approval_policy) {
    Ok(policy) => policy,
    Err(error) => return Err(ConfigError::InvalidExecutionPolicy(error)),
  };

  return Ok(policy);
}

pub fn initialize(path: &Path) -> Result<(), ConfigError> {
  let configuration = Configuration {
    profiles: HashMap::new(),
    execution: ExecutionConfiguration::safe_default(),
  };
  let parent = match path.parent() {
    Some(parent) => parent,
    None => return Err(ConfigError::Unwritable),
  };
  if fs::create_dir_all(parent).is_err() {
    return Err(ConfigError::Unwritable);
  }
  let contents = configuration.render();
  let mut file = match OpenOptions::new().write(true).create_new(true).open(path) {
    Ok(file) => file,
    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
      return Err(ConfigError::AlreadyExists);
    }
    Err(_) => return Err(ConfigError::Unwritable),
  };
  if file.write_all(contents.as_bytes()).is_err() || file.sync_all().is_err() {
    return Err(ConfigError::Unwritable);
  }

  return Ok(());
}

pub fn get(path: &Path, key: &str) -> Result<Option<String>, ConfigError> {
  let key = match ConfigKey::parse(key) {
    Ok(key) => key,
    Err(error) => return Err(error),
  };
  let configuration = match Configuration::load(path) {
    Ok(configuration) => configuration,
    Err(error) => return Err(error),
  };
  let value = match key {
    ConfigKey::Profile(key) => match configuration.profiles.get(&key.profile) {
      Some(profile) => profile.get(key.field),
      None => None,
    },
    ConfigKey::Execution(field) => configuration.execution.get(field),
  };

  return Ok(value);
}

pub fn set(path: &Path, key: &str, value: String) -> Result<(), ConfigError> {
  let key = match ConfigKey::parse(key) {
    Ok(key) => key,
    Err(error) => return Err(error),
  };
  let mut configuration = match Configuration::load(path) {
    Ok(configuration) => configuration,
    Err(error) => return Err(error),
  };
  let result = match key {
    ConfigKey::Profile(key) => {
      let profile = configuration.profiles.entry(key.profile).or_default();

      profile.set(key.field, value)
    }
    ConfigKey::Execution(field) => configuration.execution.set(field, value),
  };

  match result {
    Ok(()) => {}
    Err(error) => return Err(error),
  }

  return configuration.save(path);
}

pub fn unset(path: &Path, key: &str) -> Result<bool, ConfigError> {
  let key = match ConfigKey::parse(key) {
    Ok(key) => key,
    Err(error) => return Err(error),
  };
  let mut configuration = match Configuration::load(path) {
    Ok(configuration) => configuration,
    Err(error) => return Err(error),
  };
  let removed = match key {
    ConfigKey::Profile(ref key) => match configuration.profiles.get_mut(&key.profile) {
      Some(profile) => profile.unset(key.field),
      None => false,
    },
    ConfigKey::Execution(field) => configuration.execution.unset(field),
  };
  if let ConfigKey::Profile(key) = key {
    let remove_profile = configuration
      .profiles
      .get(&key.profile)
      .is_some_and(ProfileConfiguration::is_empty);

    if remove_profile {
      configuration.profiles.remove(&key.profile);
    }
  }
  if removed {
    match configuration.save(path) {
      Ok(()) => {}
      Err(error) => return Err(error),
    }
  }

  return Ok(removed);
}

struct Configuration {
  profiles: HashMap<String, ProfileConfiguration>,
  execution: ExecutionConfiguration,
}

impl Configuration {
  fn load(path: &Path) -> Result<Self, ConfigError> {
    let contents = match fs::read_to_string(path) {
      Ok(contents) => contents,
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
        return Ok(Self {
          profiles: HashMap::new(),
          execution: ExecutionConfiguration::default(),
        });
      }
      Err(_) => return Err(ConfigError::Unreadable),
    };
    let document = match toml::from_str::<ConfigurationDocument>(&contents) {
      Ok(document) => document,
      Err(_) => return Err(ConfigError::InvalidDocument),
    };

    return Ok(Self {
      profiles: document.profiles,
      execution: document.execution,
    });
  }

  fn save(&self, path: &Path) -> Result<(), ConfigError> {
    let parent = match path.parent() {
      Some(parent) => parent,
      None => return Err(ConfigError::Unwritable),
    };
    if fs::create_dir_all(parent).is_err() {
      return Err(ConfigError::Unwritable);
    }
    let contents = self.render();
    let (temporary_path, mut temporary_file) = match open_temporary_file(parent) {
      Ok(temporary_file) => temporary_file,
      Err(error) => return Err(error),
    };
    if temporary_file.write_all(contents.as_bytes()).is_err() || temporary_file.sync_all().is_err()
    {
      match fs::remove_file(&temporary_path) {
        Ok(()) | Err(_) => {}
      }
      return Err(ConfigError::Unwritable);
    }
    if fs::rename(&temporary_path, path).is_err() {
      match fs::remove_file(&temporary_path) {
        Ok(()) | Err(_) => {}
      }
      return Err(ConfigError::Unwritable);
    }

    return Ok(());
  }

  fn render(&self) -> String {
    let mut profiles = self.profiles.iter().collect::<Vec<_>>();
    profiles.sort_by(|left, right| return left.0.cmp(right.0));
    let mut contents =
      String::from("# Kermet configuration. Credentials are stored in the system keyring.\n");

    if !self.execution.is_empty() {
      contents.push_str("\n[execution]\n");
      self.execution.write_to(&mut contents);
    }

    for (profile_name, profile) in profiles {
      contents.push_str(&format!("\n[profiles.{profile_name}]\n"));
      profile.write_to(&mut contents);
    }

    return contents;
  }
}

const TEMPORARY_FILE_ATTEMPTS: usize = 8;

fn open_temporary_file(parent: &Path) -> Result<(PathBuf, File), ConfigError> {
  for _ in 0..TEMPORARY_FILE_ATTEMPTS {
    let nonce = rand::random::<u64>();
    let path = parent.join(format!(".config-{}-{nonce}.tmp", std::process::id()));
    let file = match OpenOptions::new().write(true).create_new(true).open(&path) {
      Ok(file) => file,
      Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
      Err(_) => return Err(ConfigError::Unwritable),
    };

    return Ok((path, file));
  }

  return Err(ConfigError::Unwritable);
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigurationDocument {
  #[serde(default)]
  profiles: HashMap<String, ProfileConfiguration>,
  #[serde(default)]
  execution: ExecutionConfiguration,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionConfiguration {
  sandbox: Option<String>,
  network: Option<String>,
  approval_policy: Option<String>,
}

impl ExecutionConfiguration {
  fn safe_default() -> Self {
    return Self {
      sandbox: Some("read-only".to_owned()),
      network: Some("denied".to_owned()),
      approval_policy: Some("on-request".to_owned()),
    };
  }

  fn sandbox_mode(&self) -> Option<SandboxMode> {
    let mode = self
      .sandbox
      .as_deref()
      .and_then(SandboxMode::from_identifier);

    return mode;
  }

  fn network_access(&self) -> Option<NetworkAccess> {
    let access = self
      .network
      .as_deref()
      .and_then(NetworkAccess::from_identifier);

    return access;
  }

  fn approval_policy(&self) -> Option<ApprovalPolicy> {
    let policy = self
      .approval_policy
      .as_deref()
      .and_then(ApprovalPolicy::from_identifier);

    return policy;
  }

  fn get(&self, field: ExecutionField) -> Option<String> {
    let value = match field {
      ExecutionField::Sandbox => self.sandbox.clone(),
      ExecutionField::Network => self.network.clone(),
      ExecutionField::ApprovalPolicy => self.approval_policy.clone(),
    };

    return value;
  }

  fn set(&mut self, field: ExecutionField, value: String) -> Result<(), ConfigError> {
    let is_valid = match field {
      ExecutionField::Sandbox => SandboxMode::from_identifier(&value).is_some(),
      ExecutionField::Network => NetworkAccess::from_identifier(&value).is_some(),
      ExecutionField::ApprovalPolicy => ApprovalPolicy::from_identifier(&value).is_some(),
    };
    if !is_valid {
      return Err(ConfigError::InvalidValue);
    }
    match field {
      ExecutionField::Sandbox => self.sandbox = Some(value),
      ExecutionField::Network => self.network = Some(value),
      ExecutionField::ApprovalPolicy => self.approval_policy = Some(value),
    }

    return Ok(());
  }

  fn unset(&mut self, field: ExecutionField) -> bool {
    let removed = match field {
      ExecutionField::Sandbox => self.sandbox.take().is_some(),
      ExecutionField::Network => self.network.take().is_some(),
      ExecutionField::ApprovalPolicy => self.approval_policy.take().is_some(),
    };

    return removed;
  }

  fn is_empty(&self) -> bool {
    return self.sandbox.is_none() && self.network.is_none() && self.approval_policy.is_none();
  }

  fn write_to(&self, contents: &mut String) -> () {
    if let Some(sandbox) = self.sandbox.as_ref() {
      contents.push_str(&format!(
        "sandbox = {}\n",
        toml::Value::String(sandbox.clone())
      ));
    }
    if let Some(network) = self.network.as_ref() {
      contents.push_str(&format!(
        "network = {}\n",
        toml::Value::String(network.clone())
      ));
    }
    if let Some(approval_policy) = self.approval_policy.as_ref() {
      contents.push_str(&format!(
        "approval_policy = {}\n",
        toml::Value::String(approval_policy.clone())
      ));
    }

    return;
  }
}

#[derive(Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileConfiguration {
  provider: Option<String>,
  model: Option<String>,
  reasoning_effort: Option<ReasoningEffort>,
}

impl ProfileConfiguration {
  fn get(&self, field: ProfileField) -> Option<String> {
    let value = match field {
      ProfileField::Provider => self.provider.clone(),
      ProfileField::Model => self.model.clone(),
      ProfileField::ReasoningEffort => self
        .reasoning_effort
        .map(|value| return value.identifier().to_owned()),
    };

    return value;
  }

  fn set(&mut self, field: ProfileField, value: String) -> Result<(), ConfigError> {
    match field {
      ProfileField::Provider if value.trim().is_empty() => return Err(ConfigError::InvalidValue),
      ProfileField::Model if value.trim().is_empty() => return Err(ConfigError::InvalidValue),
      ProfileField::Provider => self.provider = Some(value),
      ProfileField::Model => self.model = Some(value),
      ProfileField::ReasoningEffort => {
        let reasoning_effort = match ReasoningEffort::from_identifier(&value) {
          Some(reasoning_effort) => reasoning_effort,
          None => return Err(ConfigError::InvalidValue),
        };

        self.reasoning_effort = Some(reasoning_effort);
      }
    }

    return Ok(());
  }

  fn unset(&mut self, field: ProfileField) -> bool {
    let value = match field {
      ProfileField::Provider => self.provider.take().is_some(),
      ProfileField::Model => self.model.take().is_some(),
      ProfileField::ReasoningEffort => self.reasoning_effort.take().is_some(),
    };

    return value;
  }

  fn is_empty(&self) -> bool {
    return self.provider.is_none() && self.model.is_none() && self.reasoning_effort.is_none();
  }

  fn write_to(&self, contents: &mut String) -> () {
    if let Some(provider) = self.provider.as_ref() {
      contents.push_str(&format!(
        "provider = {}\n",
        toml::Value::String(provider.clone())
      ));
    }
    if let Some(model) = self.model.as_ref() {
      contents.push_str(&format!("model = {}\n", toml::Value::String(model.clone())));
    }
    if let Some(reasoning_effort) = self.reasoning_effort {
      let value = reasoning_effort.identifier().to_owned();
      contents.push_str(&format!(
        "reasoning_effort = {}\n",
        toml::Value::String(value)
      ));
    }

    return;
  }
}

enum ConfigKey {
  Profile(ProfileKey),
  Execution(ExecutionField),
}

impl ConfigKey {
  fn parse(value: &str) -> Result<Self, ConfigError> {
    if let Some(field) = value.strip_prefix("execution.") {
      let field = match field {
        "sandbox" => ExecutionField::Sandbox,
        "network" => ExecutionField::Network,
        "approval_policy" => ExecutionField::ApprovalPolicy,
        _ => return Err(ConfigError::InvalidKey),
      };

      return Ok(Self::Execution(field));
    }
    let key = match ProfileKey::parse(value) {
      Ok(key) => key,
      Err(error) => return Err(error),
    };

    return Ok(Self::Profile(key));
  }
}

struct ProfileKey {
  profile: String,
  field: ProfileField,
}

impl ProfileKey {
  fn parse(value: &str) -> Result<Self, ConfigError> {
    let mut parts = value.split('.');
    let section = parts.next();
    let profile = parts.next();
    let field = parts.next();

    if section != Some("profiles") || parts.next().is_some() {
      return Err(ConfigError::InvalidKey);
    }
    let profile = match profile {
      Some(profile) if is_valid_profile(profile) => profile.to_owned(),
      _ => return Err(ConfigError::InvalidKey),
    };
    let field = match field {
      Some("provider") => ProfileField::Provider,
      Some("model") => ProfileField::Model,
      Some("reasoning_effort") => ProfileField::ReasoningEffort,
      _ => return Err(ConfigError::InvalidKey),
    };

    return Ok(Self { profile, field });
  }
}

#[derive(Clone, Copy)]
enum ProfileField {
  Provider,
  Model,
  ReasoningEffort,
}

#[derive(Clone, Copy)]
enum ExecutionField {
  Sandbox,
  Network,
  ApprovalPolicy,
}

fn is_valid_profile(value: &str) -> bool {
  if value.is_empty() {
    return false;
  }

  return value.bytes().all(|byte| {
    return byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_';
  });
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConfigError {
  HomeUnavailable,
  AlreadyExists,
  Unreadable,
  Unwritable,
  InvalidDocument,
  InvalidKey,
  InvalidValue,
  MissingProvider,
  MissingModel,
  MissingReasoningEffort,
  InvalidSelection(ModelSelectionError),
  MissingSandboxMode,
  MissingNetworkAccess,
  MissingApprovalPolicy,
  InvalidExecutionPolicy(ExecutionPolicyError),
}

#[cfg(test)]
mod tests {
  use super::{
    ConfigError, ExecutionOverrides, ModelOverrides, get, initialize, resolve_execution_policy,
    resolve_model_selection, set, unset,
  };
  use crate::{
    models::ReasoningEffort,
    security::{
      ApprovalPolicy,
      sandbox::{NetworkAccess, SandboxMode},
    },
  };
  use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
  };

  static CONFIG_SEQUENCE: AtomicU64 = AtomicU64::new(0);

  fn create_config(contents: &str) -> PathBuf {
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("the system clock should be after the Unix epoch")
      .as_nanos();
    let sequence = CONFIG_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("kermet-config-{timestamp}-{sequence}.toml"));

    fs::write(&path, contents).expect("the configuration should be written");

    return path;
  }

  fn create_config_path() -> PathBuf {
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("the system clock should be after the Unix epoch")
      .as_nanos();
    let sequence = CONFIG_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory =
      std::env::temp_dir().join(format!("kermet-config-directory-{timestamp}-{sequence}"));

    fs::create_dir(&directory).expect("the configuration directory should be created");

    return directory.join("config.toml");
  }

  #[test]
  fn resolves_a_profile_model_selection() -> () {
    let path = create_config(
      "[profiles.default]\nprovider = \"codex\"\nmodel = \"gpt-6.1-sol\"\nreasoning_effort = \"medium\"\n",
    );
    let selection = resolve_model_selection(&path, "default", ModelOverrides::none())
      .expect("the profile should resolve");

    fs::remove_file(&path).expect("the configuration should be removed");

    assert_eq!(selection.provider(), "codex");
    assert_eq!(selection.model(), "gpt-6.1-sol");
    assert_eq!(selection.reasoning_effort(), ReasoningEffort::Medium);

    return;
  }

  #[test]
  fn command_line_overrides_replace_profile_values() -> () {
    let path = create_config(
      "[profiles.default]\nprovider = \"codex\"\nmodel = \"gpt-6.1-sol\"\nreasoning_effort = \"medium\"\n",
    );
    let overrides = ModelOverrides {
      provider: None,
      model: Some("gpt-6-luna".to_owned()),
      reasoning_effort: Some(ReasoningEffort::High),
    };
    let selection =
      resolve_model_selection(&path, "default", overrides).expect("the overrides should resolve");

    fs::remove_file(&path).expect("the configuration should be removed");

    assert_eq!(selection.model(), "gpt-6-luna");
    assert_eq!(selection.reasoning_effort(), ReasoningEffort::High);

    return;
  }

  #[test]
  fn resolves_the_global_execution_policy() -> () {
    let path = create_config(
      "[execution]\nsandbox = \"workspace-write\"\nnetwork = \"denied\"\napproval_policy = \"on-request\"\n",
    );
    let policy = resolve_execution_policy(&path, ExecutionOverrides::none())
      .expect("the execution policy should resolve");

    fs::remove_file(&path).expect("the configuration should be removed");

    assert_eq!(policy.sandbox_mode(), SandboxMode::WorkspaceWrite);
    assert_eq!(policy.network_access(), NetworkAccess::Denied);
    assert_eq!(policy.approval_policy(), ApprovalPolicy::OnRequest);

    return;
  }

  #[test]
  fn rejects_a_selection_without_a_model() -> () {
    let path =
      create_config("[profiles.default]\nprovider = \"codex\"\nreasoning_effort = \"medium\"\n");
    let error = resolve_model_selection(&path, "default", ModelOverrides::none())
      .expect_err("the selection should be rejected");

    fs::remove_file(&path).expect("the configuration should be removed");

    assert_eq!(error, ConfigError::MissingModel);

    return;
  }

  #[test]
  fn manages_profile_configuration_values() -> () {
    let path = create_config_path();

    initialize(&path).expect("the configuration should be initialized");
    set(&path, "profiles.default.provider", "codex".to_owned())
      .expect("the provider should be saved");
    set(&path, "profiles.default.model", "gpt-6.1-sol".to_owned())
      .expect("the model should be saved");
    set(
      &path,
      "profiles.default.reasoning_effort",
      "medium".to_owned(),
    )
    .expect("the reasoning effort should be saved");
    set(&path, "execution.sandbox", "workspace-write".to_owned())
      .expect("the sandbox should be saved");
    set(&path, "execution.network", "denied".to_owned())
      .expect("the network access should be saved");
    set(&path, "execution.approval_policy", "on-request".to_owned())
      .expect("the approval policy should be saved");
    let model = get(&path, "profiles.default.model")
      .expect("the model should be read")
      .expect("the model should exist");
    let removed = unset(&path, "profiles.default.reasoning_effort")
      .expect("the reasoning effort should be removed");
    let sandbox = get(&path, "execution.sandbox")
      .expect("the sandbox should be read")
      .expect("the sandbox should exist");

    fs::remove_dir_all(path.parent().expect("the path should have a parent"))
      .expect("the configuration directory should be removed");

    assert_eq!(model, "gpt-6.1-sol");
    assert_eq!(sandbox, "workspace-write");
    assert!(removed);

    return;
  }

  #[test]
  fn preserves_an_existing_configuration_when_initialization_is_repeated() -> () {
    let path = create_config("[profiles.default]\nprovider = \"codex\"\n");
    let error = initialize(&path).expect_err("the existing configuration should be preserved");
    let contents = fs::read_to_string(&path).expect("the existing configuration should be read");

    fs::remove_file(&path).expect("the configuration should be removed");

    assert_eq!(error, ConfigError::AlreadyExists);
    assert_eq!(contents, "[profiles.default]\nprovider = \"codex\"\n");

    return;
  }
}
