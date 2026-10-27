pub mod auth;

use crate::{
  models::{
    ModelEvent, ModelProvider, ModelProviderError, ModelRequest, ModelResponse, ModelToolCall,
  },
  tools::{BashRequest, EditRequest, ReadFileRequest, ToolRequest, WriteRequest},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use std::{
  io::{BufRead, BufReader},
  path::PathBuf,
  time::Duration,
};

const BASE_URL: &str = "https://chatgpt.com/backend-api/codex/responses";
const RESPONSE_LIMIT: u64 = 2 * 1024 * 1024;

pub struct CodexProvider {
  access_token: String,
  agent: ureq::Agent,
}

impl CodexProvider {
  pub fn new(access_token: String) -> Self {
    let config = ureq::Agent::config_builder()
      .https_only(true)
      .timeout_global(Some(Duration::from_secs(120)))
      .build();

    return Self {
      access_token,
      agent: config.new_agent(),
    };
  }

  pub fn is_configured(&self) -> bool {
    return !self.access_token.is_empty();
  }

  fn body(&self, request: &ModelRequest) -> Value {
    let input = match request.tool_output() {
      Some(output) => {
        let call = match request.tool_call() {
          Some(call) => call,
          None => return json!({}),
        };
        vec![
          json!({"role":"user","content":request.task()}),
          json!({"type":"function_call","call_id":call.id(),"name":call.tool_name().function_name(),"arguments":call.arguments()}),
          json!({"type":"function_call_output","call_id":call.id(),"output":output.model_content()}),
        ]
      }
      None => vec![json!({"role":"user","content":request.task()})],
    };
    let selection = request.model_selection();

    return json!({
      "model": selection.model(), "input": input,
      "instructions": "You are Kermet, a coding agent. Use only the provided tools when needed.",
      "store": false, "stream": true, "tool_choice": "auto", "parallel_tool_calls": false,
      "reasoning": {"effort": selection.reasoning_effort().identifier()},
      "tools": [
        {"type":"function","name":"read_file","description":"Read a file within the workspace.","parameters":{"type":"object","properties":{"path":{"type":"string"},"start_line":{"type":"integer","minimum":1},"end_line":{"type":"integer","minimum":1}},"required":["path"],"additionalProperties":false}},
        {"type":"function","name":"write","description":"Create or atomically replace a complete file within the workspace. This requires workspace-write or expanded access.","parameters":{"type":"object","properties":{"path":{"type":"string","minLength":1},"content":{"type":"string","maxLength":262144}},"required":["path","content"],"additionalProperties":false}},
        {"type":"function","name":"edit","description":"Atomically replace one exact text occurrence in an existing workspace file. The old_text must occur exactly once. This requires workspace-write or expanded access.","parameters":{"type":"object","properties":{"path":{"type":"string","minLength":1},"old_text":{"type":"string","minLength":1,"maxLength":262144},"new_text":{"type":"string","maxLength":262144}},"required":["path","old_text","new_text"],"additionalProperties":false}},
        {"type":"function","name":"bash","description":"Run a Bash command in the workspace. This requires expanded access.","parameters":{"type":"object","properties":{"command":{"type":"string","minLength":1,"maxLength":16384}},"required":["command"],"additionalProperties":false}}
      ],
    });
  }

  fn send(
    &self,
    body: Value,
    on_event: &mut dyn FnMut(ModelEvent),
  ) -> Result<ModelResponse, ModelProviderError> {
    let body = match serde_json::to_string(&body) {
      Ok(body) => body,
      Err(_) => return Err(ModelProviderError::InvalidResponse),
    };
    let mut request = self
      .agent
      .post(BASE_URL)
      .header("Authorization", &format!("Bearer {}", self.access_token))
      .header("Content-Type", "application/json")
      .header("User-Agent", concat!("kermet/", env!("CARGO_PKG_VERSION")))
      .header("originator", "kermet");
    for (name, value) in account_headers(&self.access_token) {
      request = request.header(&name, &value);
    }
    let mut response = match request.send(body) {
      Ok(response) => response,
      Err(_) => return Err(ModelProviderError::RequestFailed),
    };
    let reader = response
      .body_mut()
      .with_config()
      .limit(RESPONSE_LIMIT)
      .reader();
    let mut reader = BufReader::new(reader);
    let mut output = Vec::new();
    loop {
      let mut line = String::new();
      let bytes_read = match reader.read_line(&mut line) {
        Ok(bytes_read) => bytes_read,
        Err(_) => return Err(ModelProviderError::RequestFailed),
      };
      if bytes_read == 0 {
        break;
      }
      match parse_sse_line(&line, &mut output, on_event) {
        Ok(()) => (),
        Err(error) => return Err(error),
      };
    }
    let response = match parse_output(&output) {
      Ok(response) => response,
      Err(error) => return Err(error),
    };

    return Ok(response);
  }
}

fn account_headers(access_token: &str) -> Vec<(String, String)> {
  let mut headers = Vec::new();
  let payload = match access_token.split('.').nth(1) {
    Some(payload) => payload,
    None => return headers,
  };
  let decoded = match URL_SAFE_NO_PAD.decode(payload) {
    Ok(decoded) => decoded,
    Err(_) => return headers,
  };
  let claims = match serde_json::from_slice::<Value>(&decoded) {
    Ok(claims) => claims,
    Err(_) => return headers,
  };
  let auth = match claims.get("https://api.openai.com/auth") {
    Some(auth) => auth,
    None => return headers,
  };
  if let Some(account_id) = auth.get("chatgpt_account_id").and_then(Value::as_str) {
    headers.push(("ChatGPT-Account-ID".to_owned(), account_id.to_owned()));
  }
  if let Some(residency) = auth
    .get("chatgpt_data_residency")
    .or_else(|| return auth.get("chatgpt_compute_residency"))
    .and_then(Value::as_str)
  {
    headers.push((
      "x-openai-internal-codex-residency".to_owned(),
      residency.to_owned(),
    ));
  }

  return headers;
}

impl ModelProvider for CodexProvider {
  fn respond(
    &self,
    request: ModelRequest,
    on_event: &mut dyn FnMut(ModelEvent),
  ) -> Result<ModelResponse, ModelProviderError> {
    if !self.is_configured() {
      return Err(ModelProviderError::MissingCredentials);
    }
    let response = match self.send(self.body(&request), on_event) {
      Ok(response) => response,
      Err(error) => return Err(error),
    };

    return Ok(response);
  }
}

#[cfg(test)]
fn parse_sse(content: &str) -> Result<ModelResponse, ModelProviderError> {
  let mut output = Vec::new();
  let mut on_event = |_| ();
  for line in content.lines() {
    match parse_sse_line(line, &mut output, &mut on_event) {
      Ok(()) => (),
      Err(error) => return Err(error),
    };
  }

  return parse_output(&output);
}

fn parse_sse_line(
  line: &str,
  output: &mut Vec<Value>,
  on_event: &mut dyn FnMut(ModelEvent),
) -> Result<(), ModelProviderError> {
  let Some(data) = line.strip_prefix("data: ") else {
    return Ok(());
  };
  let event = match serde_json::from_str::<Value>(data) {
    Ok(event) => event,
    Err(_) if data.trim() == "[DONE]" => return Ok(()),
    Err(_) => return Err(ModelProviderError::InvalidResponse),
  };
  match event.get("type").and_then(Value::as_str) {
    Some("response.output_item.added")
      if event.pointer("/item/type").and_then(Value::as_str) == Some("reasoning") =>
    {
      on_event(ModelEvent::ThinkingStarted);
    }
    Some("response.output_text.delta") => {
      let delta = match event.get("delta").and_then(Value::as_str) {
        Some(delta) => delta.to_owned(),
        None => return Err(ModelProviderError::InvalidResponse),
      };
      on_event(ModelEvent::TextDelta(delta));
    }
    Some("response.completed") => {
      *output = match event.pointer("/response/output").and_then(Value::as_array) {
        Some(output) => output.clone(),
        None => return Err(ModelProviderError::InvalidResponse),
      };
    }
    _ => (),
  }

  return Ok(());
}

fn parse_output(output: &[Value]) -> Result<ModelResponse, ModelProviderError> {
  for item in output {
    if item.get("type").and_then(Value::as_str) == Some("function_call") {
      let id = match item.get("call_id").and_then(Value::as_str) {
        Some(id) => id.to_owned(),
        None => return Err(ModelProviderError::InvalidResponse),
      };
      let arguments = match item.get("arguments").and_then(Value::as_str) {
        Some(arguments) => arguments,
        None => return Err(ModelProviderError::InvalidResponse),
      };
      let request = match item.get("name").and_then(Value::as_str) {
        Some("bash") => match parse_bash_request(arguments) {
          Ok(request) => ToolRequest::Bash(request),
          Err(error) => return Err(error),
        },
        Some("edit") => match parse_edit_request(arguments) {
          Ok(request) => ToolRequest::Edit(request),
          Err(error) => return Err(error),
        },
        Some("read_file") => match parse_read_file_request(arguments) {
          Ok(request) => ToolRequest::ReadFile(request),
          Err(error) => return Err(error),
        },
        Some("write") => match parse_write_request(arguments) {
          Ok(request) => ToolRequest::Write(request),
          Err(error) => return Err(error),
        },
        _ => return Err(ModelProviderError::UnsupportedToolCall),
      };
      return Ok(ModelResponse::ToolCall(ModelToolCall::new(
        id,
        request,
        arguments.to_owned(),
      )));
    }
  }
  let text = output
    .iter()
    .filter(|item| return item.get("type").and_then(Value::as_str) == Some("message"))
    .flat_map(|item| {
      return item
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    })
    .filter_map(|part| return part.get("text").and_then(Value::as_str))
    .collect::<String>();
  if text.is_empty() {
    return Err(ModelProviderError::InvalidResponse);
  }

  return Ok(ModelResponse::Final(text));
}

fn parse_read_file_request(arguments: &str) -> Result<ReadFileRequest, ModelProviderError> {
  let value = match serde_json::from_str::<Value>(arguments) {
    Ok(value) => value,
    Err(_) => return Err(ModelProviderError::InvalidResponse),
  };
  let object = match value.as_object() {
    Some(object) => object,
    None => return Err(ModelProviderError::InvalidResponse),
  };
  if !object
    .keys()
    .all(|key| return matches!(key.as_str(), "path" | "start_line" | "end_line"))
  {
    return Err(ModelProviderError::InvalidResponse);
  }
  let path = match object.get("path").and_then(Value::as_str) {
    Some(path) if !path.is_empty() => PathBuf::from(path),
    _ => return Err(ModelProviderError::InvalidResponse),
  };
  let start_line = match object.get("start_line") {
    Some(value) => match parse_line_number(value) {
      Ok(line) => Some(line),
      Err(error) => return Err(error),
    },
    None => None,
  };
  let end_line = match object.get("end_line") {
    Some(value) => match parse_line_number(value) {
      Ok(line) => Some(line),
      Err(error) => return Err(error),
    },
    None => None,
  };
  let request = match (start_line, end_line) {
    (None, None) => ReadFileRequest::new(path),
    (Some(start), Some(end)) => match ReadFileRequest::with_line_range(path, start, end) {
      Ok(request) => request,
      Err(_) => return Err(ModelProviderError::InvalidResponse),
    },
    _ => return Err(ModelProviderError::InvalidResponse),
  };

  return Ok(request);
}

fn parse_bash_request(arguments: &str) -> Result<BashRequest, ModelProviderError> {
  let value = match serde_json::from_str::<Value>(arguments) {
    Ok(value) => value,
    Err(_) => return Err(ModelProviderError::InvalidResponse),
  };
  let object = match value.as_object() {
    Some(object) => object,
    None => return Err(ModelProviderError::InvalidResponse),
  };
  if object.len() != 1 || !object.contains_key("command") {
    return Err(ModelProviderError::InvalidResponse);
  }
  let command = match object.get("command").and_then(Value::as_str) {
    Some(command) => command.to_owned(),
    None => return Err(ModelProviderError::InvalidResponse),
  };
  let request = match BashRequest::new(command) {
    Ok(request) => request,
    Err(_) => return Err(ModelProviderError::InvalidResponse),
  };

  return Ok(request);
}

fn parse_edit_request(arguments: &str) -> Result<EditRequest, ModelProviderError> {
  let value = match serde_json::from_str::<Value>(arguments) {
    Ok(value) => value,
    Err(_) => return Err(ModelProviderError::InvalidResponse),
  };
  let object = match value.as_object() {
    Some(object) => object,
    None => return Err(ModelProviderError::InvalidResponse),
  };
  if object.len() != 3
    || !object.contains_key("path")
    || !object.contains_key("old_text")
    || !object.contains_key("new_text")
  {
    return Err(ModelProviderError::InvalidResponse);
  }
  let path = match object.get("path").and_then(Value::as_str) {
    Some(path) => PathBuf::from(path),
    None => return Err(ModelProviderError::InvalidResponse),
  };
  let old_text = match object.get("old_text").and_then(Value::as_str) {
    Some(old_text) => old_text.to_owned(),
    None => return Err(ModelProviderError::InvalidResponse),
  };
  let new_text = match object.get("new_text").and_then(Value::as_str) {
    Some(new_text) => new_text.to_owned(),
    None => return Err(ModelProviderError::InvalidResponse),
  };
  let request = match EditRequest::new(path, old_text, new_text) {
    Ok(request) => request,
    Err(_) => return Err(ModelProviderError::InvalidResponse),
  };

  return Ok(request);
}

fn parse_write_request(arguments: &str) -> Result<WriteRequest, ModelProviderError> {
  let value = match serde_json::from_str::<Value>(arguments) {
    Ok(value) => value,
    Err(_) => return Err(ModelProviderError::InvalidResponse),
  };
  let object = match value.as_object() {
    Some(object) => object,
    None => return Err(ModelProviderError::InvalidResponse),
  };
  if object.len() != 2 || !object.contains_key("path") || !object.contains_key("content") {
    return Err(ModelProviderError::InvalidResponse);
  }
  let path = match object.get("path").and_then(Value::as_str) {
    Some(path) => PathBuf::from(path),
    None => return Err(ModelProviderError::InvalidResponse),
  };
  let content = match object.get("content").and_then(Value::as_str) {
    Some(content) => content.to_owned(),
    None => return Err(ModelProviderError::InvalidResponse),
  };
  let request = match WriteRequest::new(path, content) {
    Ok(request) => request,
    Err(_) => return Err(ModelProviderError::InvalidResponse),
  };

  return Ok(request);
}

fn parse_line_number(value: &Value) -> Result<usize, ModelProviderError> {
  let line = match value.as_u64() {
    Some(line) => line,
    None => return Err(ModelProviderError::InvalidResponse),
  };
  let line = match usize::try_from(line) {
    Ok(line) => line,
    Err(_) => return Err(ModelProviderError::InvalidResponse),
  };
  if line == 0 {
    return Err(ModelProviderError::InvalidResponse);
  }

  return Ok(line);
}

#[cfg(test)]
mod tests {
  use super::{parse_sse, parse_sse_line};
  use crate::{
    models::{ModelEvent, ModelResponse},
    tools::ToolRequest,
  };

  #[test]
  fn parses_a_final_message() -> () {
    let response = parse_sse(
      r#"data: {"type":"response.completed","response":{"output":[{"type":"message","content":[{"type":"output_text","text":"The package is Kermet."}]}]}}
"#,
    )
    .expect("the completed response should be valid");

    let ModelResponse::Final(message) = response else {
      panic!("the response should be final");
    };

    assert_eq!(message, "The package is Kermet.");

    return;
  }

  #[test]
  fn parses_and_preserves_a_read_file_call() -> () {
    let arguments = r#"{"path":"src/main.rs","start_line":2,"end_line":4}"#;
    let response = parse_sse(
      r#"data: {"type":"response.completed","response":{"output":[{"type":"function_call","call_id":"call_123","name":"read_file","arguments":"{\"path\":\"src/main.rs\",\"start_line\":2,\"end_line\":4}"}]}}
"#,
    )
    .expect("the completed response should be valid");

    let ModelResponse::ToolCall(call) = response else {
      panic!("the response should request a tool");
    };
    let (id, request, preserved_arguments) = call.into_parts();
    let ToolRequest::ReadFile(request) = request else {
      panic!("the request should target read_file");
    };

    assert_eq!(id, "call_123");
    assert_eq!(preserved_arguments, arguments);
    assert_eq!(request.path().to_string_lossy(), "src/main.rs");
    assert_eq!(request.line_range(), Some((2, 4)));

    return;
  }

  #[test]
  fn parses_and_preserves_a_bash_call() -> () {
    let arguments = r#"{"command":"cargo test"}"#;
    let response = parse_sse(
      r#"data: {"type":"response.completed","response":{"output":[{"type":"function_call","call_id":"call_123","name":"bash","arguments":"{\"command\":\"cargo test\"}"}]}}
"#,
    )
    .expect("the completed response should be valid");

    let ModelResponse::ToolCall(call) = response else {
      panic!("the response should request a tool");
    };
    let (id, request, preserved_arguments) = call.into_parts();
    let ToolRequest::Bash(request) = request else {
      panic!("the request should target Bash");
    };

    assert_eq!(id, "call_123");
    assert_eq!(preserved_arguments, arguments);
    assert_eq!(request.command(), "cargo test");

    return;
  }

  #[test]
  fn parses_and_preserves_a_write_call() -> () {
    let arguments = r#"{"path":"notes.txt","content":"Kermet"}"#;
    let response = parse_sse(
      r#"data: {"type":"response.completed","response":{"output":[{"type":"function_call","call_id":"call_123","name":"write","arguments":"{\"path\":\"notes.txt\",\"content\":\"Kermet\"}"}]}}
"#,
    )
    .expect("the completed response should be valid");

    let ModelResponse::ToolCall(call) = response else {
      panic!("the response should request a tool");
    };
    let (id, request, preserved_arguments) = call.into_parts();
    let ToolRequest::Write(request) = request else {
      panic!("the request should target write");
    };

    assert_eq!(id, "call_123");
    assert_eq!(preserved_arguments, arguments);
    assert_eq!(request.path().to_string_lossy(), "notes.txt");
    assert_eq!(request.contents(), "Kermet");

    return;
  }

  #[test]
  fn parses_and_preserves_an_edit_call() -> () {
    let arguments = r#"{"path":"notes.txt","old_text":"old","new_text":"new"}"#;
    let response = parse_sse(
      r#"data: {"type":"response.completed","response":{"output":[{"type":"function_call","call_id":"call_123","name":"edit","arguments":"{\"path\":\"notes.txt\",\"old_text\":\"old\",\"new_text\":\"new\"}"}]}}
"#,
    )
    .expect("the completed response should be valid");

    let ModelResponse::ToolCall(call) = response else {
      panic!("the response should request a tool");
    };
    let (id, request, preserved_arguments) = call.into_parts();
    let ToolRequest::Edit(request) = request else {
      panic!("the request should target edit");
    };

    assert_eq!(id, "call_123");
    assert_eq!(preserved_arguments, arguments);
    assert_eq!(request.path().to_string_lossy(), "notes.txt");
    assert_eq!(request.old_text(), "old");
    assert_eq!(request.new_text(), "new");

    return;
  }

  #[test]
  fn rejects_a_partial_line_range() -> () {
    let response = parse_sse(
      r#"data: {"type":"response.completed","response":{"output":[{"type":"function_call","call_id":"call_123","name":"read_file","arguments":"{\"path\":\"src/main.rs\",\"start_line\":2}"}]}}
"#,
    );

    assert!(response.is_err());

    return;
  }

  #[test]
  fn rejects_unrecognized_read_file_arguments() -> () {
    let response = parse_sse(
      r#"data: {"type":"response.completed","response":{"output":[{"type":"function_call","call_id":"call_123","name":"read_file","arguments":"{\"path\":\"src/main.rs\",\"encoding\":\"utf-8\"}"}]}}
"#,
    );

    assert!(response.is_err());

    return;
  }

  #[test]
  fn rejects_unrecognized_bash_arguments() -> () {
    let response = parse_sse(
      r#"data: {"type":"response.completed","response":{"output":[{"type":"function_call","call_id":"call_123","name":"bash","arguments":"{\"command\":\"pwd\",\"cwd\":\".\"}"}]}}
"#,
    );

    assert!(response.is_err());

    return;
  }

  #[test]
  fn rejects_unrecognized_write_arguments() -> () {
    let response = parse_sse(
      r#"data: {"type":"response.completed","response":{"output":[{"type":"function_call","call_id":"call_123","name":"write","arguments":"{\"path\":\"notes.txt\",\"content\":\"Kermet\",\"append\":true}"}]}}
"#,
    );

    assert!(response.is_err());

    return;
  }

  #[test]
  fn rejects_unrecognized_edit_arguments() -> () {
    let response = parse_sse(
      r#"data: {"type":"response.completed","response":{"output":[{"type":"function_call","call_id":"call_123","name":"edit","arguments":"{\"path\":\"notes.txt\",\"old_text\":\"old\",\"new_text\":\"new\",\"replace_all\":true}"}]}}
"#,
    );

    assert!(response.is_err());

    return;
  }

  #[test]
  fn emits_thinking_and_text_events_while_parsing() -> () {
    let mut output = Vec::new();
    let mut events = Vec::new();
    let mut on_event = |event: ModelEvent| match event {
      ModelEvent::ThinkingStarted => events.push("thinking".to_owned()),
      ModelEvent::TextDelta(delta) => events.push(format!("text:{delta}")),
    };
    let lines = [
      r#"data: {"type":"response.output_item.added","item":{"type":"reasoning"}}"#,
      r#"data: {"type":"response.output_text.delta","delta":"Kermet"}"#,
    ];

    for line in lines {
      parse_sse_line(line, &mut output, &mut on_event)
        .expect("the streaming event should be valid");
    }

    assert_eq!(events, ["thinking", "text:Kermet"]);

    return;
  }
}
