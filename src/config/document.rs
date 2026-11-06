use std::path::Path;

use toml_edit::{DocumentMut, Item, Table, value as toml_value};

use super::{ConfigError, path};

pub(crate) fn get(key: &str) -> Result<String, ConfigError> {
  let path = match path::user_path() {
    Ok(path) => path,
    Err(error) => return Err(error),
  };
  let result = get_at(&path, key);

  return result;
}

pub(crate) fn set(key: &str, value: &str) -> Result<(), ConfigError> {
  let path = match path::user_path() {
    Ok(path) => path,
    Err(error) => return Err(error),
  };
  let result = set_at(&path, key, value);

  return result;
}

pub(crate) fn unset(key: &str) -> Result<(), ConfigError> {
  let path = match path::user_path() {
    Ok(path) => path,
    Err(error) => return Err(error),
  };
  let result = unset_at(&path, key);

  return result;
}

pub(super) fn get_at(path: &Path, key: &str) -> Result<String, ConfigError> {
  let segments = match key_segments(key) {
    Ok(segments) => segments,
    Err(error) => return Err(error),
  };

  let document = match read_document_at(path) {
    Ok(document) => document,
    Err(error) => return Err(error),
  };
  let item = match get_item(document.as_table(), &segments) {
    Ok(item) => item,
    Err(error) => return Err(error),
  };
  let value = match item.as_value() {
    Some(value) => value,
    None => return Err(ConfigError::UnsupportedValue),
  };
  let value = match value.as_str() {
    Some(value) => value,
    None => return Err(ConfigError::UnsupportedValue),
  };

  return Ok(value.to_owned());
}

pub(super) fn set_at(path: &Path, key: &str, value: &str) -> Result<(), ConfigError> {
  let segments = match key_segments(key) {
    Ok(segments) => segments,
    Err(error) => return Err(error),
  };

  let mut document = match read_document_at(path) {
    Ok(document) => document,
    Err(error) => return Err(error),
  };

  let result = set_item(document.as_table_mut(), &segments, value);

  if let Err(error) = result {
    return Err(error);
  }

  let result = write_at(path, document);

  return result;
}

pub(super) fn unset_at(path: &Path, key: &str) -> Result<(), ConfigError> {
  let segments = match key_segments(key) {
    Ok(segments) => segments,
    Err(error) => return Err(error),
  };

  let mut document = match read_document_at(path) {
    Ok(document) => document,
    Err(error) => return Err(error),
  };

  let result = unset_item(document.as_table_mut(), &segments);

  if let Err(error) = result {
    return Err(error);
  }

  let result = write_at(path, document);

  return result;
}

fn parse(contents: &str) -> Result<DocumentMut, ConfigError> {
  let document = match contents.parse::<DocumentMut>() {
    Ok(document) => document,
    Err(_) => return Err(ConfigError::InvalidDocument),
  };

  return Ok(document);
}

fn read_document_at(path: &Path) -> Result<DocumentMut, ConfigError> {
  let contents = match path::read_at(path) {
    Ok(contents) => contents,
    Err(error) => return Err(error),
  };
  let document = match parse(&contents) {
    Ok(document) => document,
    Err(error) => return Err(error),
  };

  return Ok(document);
}

fn key_segments(key: &str) -> Result<Vec<&str>, ConfigError> {
  let segments: Vec<&str> = key.split('.').collect();
  let is_valid = !key.is_empty()
    && segments
      .iter()
      .all(|segment| return !segment.is_empty() && segment.chars().all(is_key_character));

  if !is_valid {
    return Err(ConfigError::InvalidKey);
  }

  return Ok(segments);
}

fn is_key_character(character: char) -> bool {
  let is_valid = character.is_ascii_alphanumeric() || character == '_' || character == '-';

  return is_valid;
}

fn get_item<'a>(table: &'a Table, segments: &[&str]) -> Result<&'a Item, ConfigError> {
  let segment = match segments.first() {
    Some(segment) => segment,
    None => return Err(ConfigError::InvalidKey),
  };
  let item = match table.get(segment) {
    Some(item) => item,
    None => return Err(ConfigError::KeyNotFound),
  };

  if segments.len() == 1 {
    return Ok(item);
  }

  let table = match item.as_table() {
    Some(table) => table,
    None => return Err(ConfigError::PathNotTable),
  };
  let result = get_item(table, &segments[1..]);

  return result;
}

fn set_item(table: &mut Table, segments: &[&str], value: &str) -> Result<(), ConfigError> {
  let segment = match segments.first() {
    Some(segment) => segment,
    None => return Err(ConfigError::InvalidKey),
  };

  if segments.len() == 1 {
    table.insert(segment, toml_value(value));

    return Ok(());
  }

  if table.get(segment).is_none() {
    table.insert(segment, implicit_table());
  }

  let item = match table.get_mut(segment) {
    Some(item) => item,
    None => return Err(ConfigError::KeyNotFound),
  };
  let table = match item.as_table_mut() {
    Some(table) => table,
    None => return Err(ConfigError::PathNotTable),
  };
  let result = set_item(table, &segments[1..], value);

  return result;
}

fn implicit_table() -> Item {
  let mut table = Table::new();

  table.set_implicit(true);

  return Item::Table(table);
}

fn unset_item(table: &mut Table, segments: &[&str]) -> Result<(), ConfigError> {
  let segment = match segments.first() {
    Some(segment) => segment,
    None => return Err(ConfigError::InvalidKey),
  };

  if segments.len() == 1 {
    if table.remove(segment).is_none() {
      return Err(ConfigError::KeyNotFound);
    }

    return Ok(());
  }

  let item = match table.get_mut(segment) {
    Some(item) => item,
    None => return Err(ConfigError::KeyNotFound),
  };
  let table = match item.as_table_mut() {
    Some(table) => table,
    None => return Err(ConfigError::PathNotTable),
  };
  let result = unset_item(table, &segments[1..]);

  return result;
}

fn write_at(path: &Path, document: DocumentMut) -> Result<(), ConfigError> {
  if std::fs::write(path, document.to_string()).is_err() {
    return Err(ConfigError::WriteFailed);
  }

  return Ok(());
}
