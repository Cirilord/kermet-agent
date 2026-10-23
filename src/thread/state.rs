use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub(super) const SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "role", content = "content", rename_all = "lowercase")]
enum ThreadMessage {
  User { content: String },
  Assistant { content: String },
}

impl ThreadMessage {
  fn render_for_prompt(&self) -> String {
    let rendered = match *self {
      Self::User { ref content } => format!("User: {content}"),
      Self::Assistant { ref content } => format!("Assistant: {content}"),
    };

    return rendered;
  }

  fn render_for_terminal(&self) -> String {
    let rendered = match *self {
      Self::User { ref content } => format!("> {content}\n"),
      Self::Assistant { ref content } => format!("{content}\n"),
    };

    return rendered;
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Thread {
  pub(super) version: u8,
  pub(super) id: String,
  pub(super) workspace: PathBuf,
  pub(super) profile: String,
  messages: Vec<ThreadMessage>,
}

impl Thread {
  pub fn new(id: String, workspace: PathBuf, profile: String) -> Self {
    return Self {
      version: SCHEMA_VERSION,
      id,
      workspace,
      profile,
      messages: Vec::new(),
    };
  }

  pub fn id(&self) -> &str {
    return &self.id;
  }

  pub fn workspace(&self) -> &Path {
    return &self.workspace;
  }

  pub fn profile(&self) -> &str {
    return &self.profile;
  }

  pub fn message_count(&self) -> usize {
    let count = self.messages.len();

    return count;
  }

  pub fn record_user_message(&mut self, content: String) -> () {
    self.messages.push(ThreadMessage::User { content });
    return;
  }

  pub fn record_assistant_message(&mut self, content: String) -> () {
    self.messages.push(ThreadMessage::Assistant { content });
    return;
  }

  pub fn render_transcript(&self) -> String {
    let mut transcript = String::new();

    for item in &self.messages {
      transcript.push_str(&item.render_for_terminal());
    }

    return transcript;
  }

  pub fn prompt(&self, message: &str) -> String {
    if self.messages.is_empty() {
      return message.to_owned();
    }

    let mut prompt = String::from("Conversation history:\n");

    for item in &self.messages {
      prompt.push_str(&item.render_for_prompt());
      prompt.push('\n');
    }

    prompt.push_str("\nCurrent user request:\n");
    prompt.push_str(message);

    return prompt;
  }
}
