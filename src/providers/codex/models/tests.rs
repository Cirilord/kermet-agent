use super::catalog::{ModelCatalogResponse, ModelResponse, models_from_response};

#[test]
fn keeps_only_models_visible_in_the_catalog() -> () {
  let response = ModelCatalogResponse {
    models: vec![
      ModelResponse {
        display_name: "Hidden model".to_owned(),
        slug: "hidden-model".to_owned(),
        visibility: "hidden".to_owned(),
      },
      ModelResponse {
        display_name: "First model".to_owned(),
        slug: "first-model".to_owned(),
        visibility: "list".to_owned(),
      },
      ModelResponse {
        display_name: "Second model".to_owned(),
        slug: "second-model".to_owned(),
        visibility: "list".to_owned(),
      },
    ],
  };
  let models = models_from_response(response);

  assert_eq!(models.len(), 2);
  assert_eq!(models[0].display_name(), "First model");
  assert_eq!(models[0].slug(), "first-model");
  assert_eq!(models[1].display_name(), "Second model");
  assert_eq!(models[1].slug(), "second-model");

  return;
}
