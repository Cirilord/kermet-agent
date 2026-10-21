use crate::models::{ModelSelection, ModelSelectionError, ReasoningEffort};
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

pub fn initialize(path: &Path) -> Result<(), ConfigError> {
  let configuration = Configuration {
    profiles: HashMap::new(),
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
  let key = match ProfileKey::parse(key) {
    Ok(key) => key,
    Err(error) => return Err(error),
  };
  let configuration = match Configuration::load(path) {
    Ok(configuration) => configuration,
    Err(error) => return Err(error),
  };
  let value = match configuration.profiles.get(&key.profile) {
    Some(profile) => profile.get(key.field),
    None => None,
  };

  return Ok(value);
}

pub fn set(path: &Path, key: &str, value: String) -> Result<(), ConfigError> {
  let key = match ProfileKey::parse(key) {
    Ok(key) => key,
    Err(error) => return Err(error),
  };
  let mut configuration = match Configuration::load(path) {
    Ok(configuration) => configuration,
    Err(error) => return Err(error),
  };
  let profile = configuration.profiles.entry(key.profile).or_default();

  match profile.set(key.field, value) {
    Ok(()) => {}
    Err(error) => return Err(error),
  }

  return configuration.save(path);
}

pub fn unset(path: &Path, key: &str) -> Result<bool, ConfigError> {
  let key = match ProfileKey::parse(key) {
    Ok(key) => key,
    Err(error) => return Err(error),
  };
  let mut configuration = match Configuration::load(path) {
    Ok(configuration) => configuration,
    Err(error) => return Err(error),
  };
  let removed = match configuration.profiles.get_mut(&key.profile) {
    Some(profile) => profile.unset(key.field),
    None => false,
  };
  let remove_profile = configuration
    .profiles
    .get(&key.profile)
    .is_some_and(ProfileConfiguration::is_empty);

  if remove_profile {
    configuration.profiles.remove(&key.profile);
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
}

impl Configuration {
  fn load(path: &Path) -> Result<Self, ConfigError> {
    let contents = match fs::read_to_string(path) {
      Ok(contents) => contents,
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
        return Ok(Self {
          profiles: HashMap::new(),
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
}

#[cfg(test)]
mod tests {
  use super::{ConfigError, ModelOverrides, get, initialize, resolve_model_selection, set, unset};
  use crate::models::ReasoningEffort;
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
    let model = get(&path, "profiles.default.model")
      .expect("the model should be read")
      .expect("the model should exist");
    let removed = unset(&path, "profiles.default.reasoning_effort")
      .expect("the reasoning effort should be removed");

    fs::remove_dir_all(path.parent().expect("the path should have a parent"))
      .expect("the configuration directory should be removed");

    assert_eq!(model, "gpt-6.1-sol");
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
