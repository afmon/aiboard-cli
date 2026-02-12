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

    pub fn ingest(
        &self,
        thread_id: &str,
        json_input: &str,
    ) -> Result<usize, DomainError> {
        let parsed: serde_json::Value = serde_json::from_str(json_input)
            .map_err(|e| DomainError::Parse(format!("invalid JSON: {}", e)))?;

        let session_id = parsed
            .get("session_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let messages_val = parsed
            .get("messages")
            .and_then(|v| v.as_array())
            .ok_or_else(|| DomainError::Parse("missing 'messages' array".to_string()))?;

        let mut messages = Vec::new();
        for entry in messages_val {
            let role_str = entry
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("user");
            let role: Role = role_str
                .parse()
                .map_err(|e: String| DomainError::Parse(e))?;
            let content = entry
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let sender = entry
                .get("sender")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let now = Utc::now();
            messages.push(Message {
                id: Uuid::new_v4().to_string(),
                thread_id: thread_id.to_string(),
                session_id: session_id.clone(),
                sender,
                role,
                content,
                metadata: None,
                parent_id: None,
                created_at: now,
                updated_at: now,
            });
        }

        self.repo.insert_batch(&messages)
    }
}
