use super::{IdError, IdGenerator};

#[test]
fn generates_a_valid_identifier() -> () {
  let id = IdGenerator::generate("thread").expect("the identifier should be generated");

  assert!(IdGenerator::validate("thread", &id).is_ok());

  return;
}

#[test]
fn rejects_an_invalid_prefix() -> () {
  let result = IdGenerator::generate("Thread");

  assert_eq!(result, Err(IdError::InvalidPrefix));

  return;
}

#[test]
fn rejects_a_value_with_a_different_prefix() -> () {
  let result = IdGenerator::validate("thread", "evidence-0123456789abcdef");

  assert_eq!(result, Err(IdError::InvalidValue));

  return;
}
