use super::{Provider, ProviderError, ProviderKind};

#[test]
fn converts_the_codex_provider_kind_into_a_provider() -> () {
  let provider_kind = match ProviderKind::parse("codex") {
    Ok(provider_kind) => provider_kind,
    Err(_) => panic!("the Codex provider kind should parse"),
  };
  let provider = Provider::from(provider_kind);

  assert!(matches!(provider, Provider::Codex));

  return;
}

#[test]
fn rejects_an_unknown_provider_identifier() -> () {
  let result = ProviderKind::parse("unknown");

  assert!(matches!(
    result,
    Err(ProviderError::UnsupportedIdentifier(_))
  ));

  return;
}
