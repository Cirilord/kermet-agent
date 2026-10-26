use crate::{
  agent::{AgentEvent, AgentLoop, AgentLoopError, TaskRequest},
  models::ModelSelection,
  providers::{ProviderRegistry, ProviderRegistryError},
  security::{KeyringCredentialStore, PermissionManager},
  tools::ToolRegistry,
};

pub enum TurnError {
  Provider(ProviderRegistryError),
  Agent(AgentLoopError),
}

pub fn execute_turn(
  request: &TaskRequest,
  model_selection: &ModelSelection,
  profile: &str,
  on_event: &mut dyn FnMut(AgentEvent),
) -> Result<String, TurnError> {
  let registry = ProviderRegistry::new(KeyringCredentialStore::new());
  let provider = match registry.resolve(model_selection, profile) {
    Ok(provider) => provider,
    Err(error) => return Err(TurnError::Provider(error)),
  };
  let tools = ToolRegistry::new(PermissionManager::new());
  let agent_loop = AgentLoop::new(provider, tools);
  let result = match agent_loop.execute(request, model_selection, on_event) {
    Ok(result) => result,
    Err(error) => return Err(TurnError::Agent(error)),
  };

  return Ok(result.message().to_owned());
}
