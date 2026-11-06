use std::{fs, path::PathBuf};

use uuid::Uuid;

use super::{
  document::{get_at, set_at, unset_at},
  path::{ensure_exists_at, exists_at, read_at},
};

#[test]
fn gets_a_string_configuration_value() -> () {
  let path = temporary_path();

  fs::write(&path, "model = \"gpt-6.1-sol\"\n").expect("the configuration file should be created");
  let value = match get_at(&path, "model") {
    Ok(value) => value,
    Err(_) => panic!("the configuration value should be read"),
  };

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert_eq!(value, "gpt-6.1-sol");

  return;
}

#[test]
fn gets_a_string_value_from_a_configuration_table() -> () {
  let path = temporary_path();

  fs::write(&path, "[profiles.default]\nmodel = \"gpt-6.1-sol\"\n")
    .expect("the configuration file should be created");
  let value = match get_at(&path, "profiles.default.model") {
    Ok(value) => value,
    Err(_) => panic!("the configuration value should be read"),
  };

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert_eq!(value, "gpt-6.1-sol");

  return;
}

#[test]
fn sets_a_string_configuration_value() -> () {
  let path = temporary_path();

  fs::write(
    &path,
    "# Kermet configuration. Credentials are stored in the system keyring.\n\n",
  )
  .expect("the configuration file should be created");
  let result = set_at(&path, "model", "gpt-6.1-sol");
  let contents = fs::read_to_string(&path).expect("the configuration file should be readable");

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert!(result.is_ok());
  assert_eq!(
    contents,
    "# Kermet configuration. Credentials are stored in the system keyring.\n\nmodel = \"gpt-6.1-sol\"\n"
  );

  return;
}

#[test]
fn creates_tables_when_setting_a_nested_configuration_value() -> () {
  let path = temporary_path();

  fs::write(&path, "").expect("the configuration file should be created");
  let result = set_at(&path, "profiles.default.model", "gpt-6.1-sol");
  let contents = fs::read_to_string(&path).expect("the configuration file should be readable");

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert!(result.is_ok());
  assert_eq!(
    contents,
    "# Kermet configuration. Credentials are stored in the system keyring.\n\n[profiles.default]\nmodel = \"gpt-6.1-sol\"\n"
  );

  return;
}

#[test]
fn unsets_a_configuration_value() -> () {
  let path = temporary_path();

  fs::write(&path, "model = \"gpt-6.1-sol\"\n").expect("the configuration file should be created");
  let result = unset_at(&path, "model");
  let contents = fs::read_to_string(&path).expect("the configuration file should be readable");

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert!(result.is_ok());
  assert_eq!(
    contents,
    "# Kermet configuration. Credentials are stored in the system keyring.\n\n"
  );

  return;
}

#[test]
fn unsets_a_nested_configuration_value() -> () {
  let path = temporary_path();

  fs::write(
    &path,
    "[profiles.default]\nmodel = \"gpt-6.1-sol\"\nprovider = \"codex\"\n",
  )
  .expect("the configuration file should be created");
  let result = unset_at(&path, "profiles.default.model");
  let contents = fs::read_to_string(&path).expect("the configuration file should be readable");

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert!(result.is_ok());
  assert_eq!(
    contents,
    "# Kermet configuration. Credentials are stored in the system keyring.\n\n[profiles.default]\nprovider = \"codex\"\n"
  );

  return;
}

#[test]
fn rejects_a_missing_configuration_value() -> () {
  let path = temporary_path();

  fs::write(&path, "").expect("the configuration file should be created");
  let result = get_at(&path, "model");

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert!(matches!(result, Err(super::ConfigError::KeyNotFound)));

  return;
}

#[test]
fn creates_the_configuration_file_when_it_is_missing() -> () {
  let path = temporary_path();

  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");
  let result = ensure_exists_at(&path);

  let contents = fs::read_to_string(&path).expect("the configuration file should be readable");

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert!(result.is_ok());
  assert_eq!(
    contents,
    "# Kermet configuration. Credentials are stored in the system keyring.\n\n"
  );

  return;
}

#[test]
fn preserves_an_existing_configuration_file() -> () {
  let path = temporary_path();

  fs::write(&path, "[profile]\nmodel = \"gpt-6.1-sol\"\n")
    .expect("the configuration file should be created");
  let result = ensure_exists_at(&path);
  let contents = fs::read_to_string(&path).expect("the configuration file should be readable");

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert!(result.is_ok());
  assert_eq!(contents, "[profile]\nmodel = \"gpt-6.1-sol\"\n");

  return;
}

#[test]
fn returns_false_when_the_configuration_file_is_missing() -> () {
  let path = temporary_path();
  let result = match exists_at(&path) {
    Ok(result) => result,
    Err(_) => panic!("the missing configuration file should be checked"),
  };

  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert!(!result);

  return;
}

#[test]
fn returns_true_when_the_configuration_file_exists() -> () {
  let path = temporary_path();

  fs::write(&path, "").expect("the configuration file should be created");
  let result = match exists_at(&path) {
    Ok(result) => result,
    Err(_) => panic!("the configuration file should be checked"),
  };

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert!(result);

  return;
}

#[test]
fn reads_the_configuration_file() -> () {
  let path = temporary_path();

  fs::write(&path, "[profile]\nmodel = \"gpt-6.1-sol\"\n")
    .expect("the configuration file should be created");
  let contents = match read_at(&path) {
    Ok(contents) => contents,
    Err(_) => panic!("the configuration file should be read"),
  };

  fs::remove_file(&path).expect("the configuration file should be removed");
  fs::remove_dir(
    path
      .parent()
      .expect("the configuration path should have a parent"),
  )
  .expect("the configuration directory should be removed");

  assert_eq!(contents, "[profile]\nmodel = \"gpt-6.1-sol\"\n");

  return;
}

fn temporary_path() -> PathBuf {
  let directory = std::env::temp_dir().join(format!("kermet-config-{}", Uuid::new_v4()));

  fs::create_dir_all(&directory).expect("the configuration directory should be created");

  return directory.join("config.toml");
}
