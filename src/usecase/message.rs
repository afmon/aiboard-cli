use crate::domain::entity::{Message, Role};
use crate::domain::error::DomainError;
use crate::domain::repository::MessageRepository;
use chrono::Utc;
use uuid::Uuid;

pub struct MessageUseCase<R: MessageRepository> {
    pub(crate) repo: R,
}

impl<R: MessageRepository> MessageUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn post(
        &self,
        thread_id: &str,
        role: Role,
        content: &str,
        session_id: Option<&str>,
        sender: Option<&str>,
        metadata: Option<serde_json::Value>,
        parent_id: Option<&str>,
    ) -> Result<Message, DomainError> {
        let now = Utc::now();
        let msg = Message {
            id: Uuid::new_v4().to_string(),
            thread_id: thread_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            sender: sender.map(|s| s.to_string()),
            role,
            content: content.to_string(),
            metadata,
            parent_id: parent_id.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
        };
        self.repo.insert(&msg)?;
        Ok(msg)
    }

    pub fn read(&self, thread_id: &str) -> Result<Vec<Message>, DomainError> {
        self.repo.find_by_thread(thread_id)
    }

    pub fn search(
        &self,
        query: &str,
        thread_id: Option<&str>,
    ) -> Result<Vec<Message>, DomainError> {
        self.repo.search(query, thread_id)
    }

    pub fn update(&self, short_id: &str, content: &str) -> Result<String, DomainError> {
        let full_id = self.repo.resolve_short_id(short_id)?;
        self.repo.update_content(&full_id, content)?;
        Ok(full_id)
    }
}
