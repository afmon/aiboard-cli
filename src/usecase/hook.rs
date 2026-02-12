use crate::domain::entity::{Message, Role};
use crate::domain::error::DomainError;
use crate::domain::repository::MessageRepository;
use chrono::Utc;
use uuid::Uuid;

pub struct HookUseCase<R: MessageRepository> {
    pub(crate) repo: R,
}

impl<R: MessageRepository> HookUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Ingest a Claude Code hook event from stdin JSON.
    ///
    /// The JSON contains common fields (session_id, hook_event_name, etc.)
    /// plus event-specific fields. A thread_id override can be provided
    /// via CLI; otherwise session_id is used as the thread_id.
    pub fn ingest(
        &self,
        thread_id_override: Option<&str>,
        json_input: &str,
    ) -> Result<usize, DomainError> {
        let parsed: serde_json::Value = serde_json::from_str(json_input)
            .map_err(|e| DomainError::Parse(format!("invalid JSON: {}", e)))?;

        let session_id = parsed
            .get("session_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let thread_id = match thread_id_override {
            Some(tid) => tid.to_string(),
            None => session_id
                .clone()
                .ok_or_else(|| DomainError::Parse("no session_id and no --thread provided".to_string()))?,
        };

        let event_name = parsed
            .get("hook_event_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let (role, content, sender) = match event_name {
            "UserPromptSubmit" => {
                let prompt = parsed
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                (Role::User, prompt, None)
            }
            "PostToolUse" => {
                let tool_name = parsed
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown_tool");
                let tool_input = parsed
                    .get("tool_input")
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                let tool_response = parsed
                    .get("tool_response")
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                let content = format!(
                    "[{}] input: {} | response: {}",
                    tool_name, tool_input, tool_response
                );
                (Role::Tool, content, Some(tool_name.to_string()))
            }
            "Stop" => {
                let content = "[session stop]".to_string();
                (Role::Assistant, content, None)
            }
            other => {
                let content = format!("[{}] event received", other);
                (Role::System, content, None)
            }
        };

        if content.is_empty() {
            return Ok(0);
        }

        let now = Utc::now();
        let message = Message {
            id: Uuid::new_v4().to_string(),
            thread_id,
            session_id,
            sender,
            role,
            content,
            metadata: None,
            parent_id: None,
            created_at: now,
            updated_at: now,
        };

        self.repo.insert_batch(&[message])
    }
}
