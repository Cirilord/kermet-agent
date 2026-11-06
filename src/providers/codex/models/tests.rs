use super::catalog::{
  ModelCatalogResponse, ModelResponse, ReasoningLevelResponse, models_from_response,
};

#[test]
fn keeps_only_models_visible_in_the_catalog() -> () {
  let response = ModelCatalogResponse {
    models: vec![
      ModelResponse {
        context_window: 128_000,
        default_reasoning_level: "medium".to_owned(),
        display_name: "Hidden model".to_owned(),
        max_context_window: 128_000,
        slug: "hidden-model".to_owned(),
        supported_reasoning_levels: vec![ReasoningLevelResponse {
          effort: "low".to_owned(),
        }],
        visibility: "hidden".to_owned(),
      },
      ModelResponse {
        context_window: 272_000,
        default_reasoning_level: "high".to_owned(),
        display_name: "First model".to_owned(),
        max_context_window: 272_000,
        slug: "first-model".to_owned(),
        supported_reasoning_levels: vec![
          ReasoningLevelResponse {
            effort: "medium".to_owned(),
          },
          ReasoningLevelResponse {
            effort: "high".to_owned(),
          },
        ],
        visibility: "list".to_owned(),
      },
      ModelResponse {
        context_window: 128_000,
        default_reasoning_level: "medium".to_owned(),
        display_name: "Second model".to_owned(),
        max_context_window: 200_000,
        slug: "second-model".to_owned(),
        supported_reasoning_levels: vec![
          ReasoningLevelResponse {
            effort: "low".to_owned(),
          },
          ReasoningLevelResponse {
            effort: "medium".to_owned(),
          },
        ],
        visibility: "list".to_owned(),
      },
    ],
  };
  let models = models_from_response(response);

  assert_eq!(models.len(), 2);
  assert_eq!(models[0].display_name(), "First model");
  assert_eq!(models[0].slug(), "first-model");
  assert_eq!(models[0].supported_reasoning_levels(), ["medium", "high"]);
  assert_eq!(models[0].default_reasoning_level(), "high");
  assert_eq!(models[0].context_window(), 272_000);
  assert_eq!(models[0].max_context_window(), 272_000);
  assert_eq!(models[1].display_name(), "Second model");
  assert_eq!(models[1].slug(), "second-model");
  assert_eq!(models[1].supported_reasoning_levels(), ["low", "medium"]);
  assert_eq!(models[1].default_reasoning_level(), "medium");
  assert_eq!(models[1].context_window(), 128_000);
  assert_eq!(models[1].max_context_window(), 200_000);

  return;
}
