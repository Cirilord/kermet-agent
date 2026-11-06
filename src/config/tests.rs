use std::{fs, path::PathBuf};

use uuid::Uuid;

use super::path::{ensure_exists_at, exists_at, read_at};

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
  assert!(contents.is_empty());

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
