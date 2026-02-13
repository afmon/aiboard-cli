use crate::domain::entity::{Message, Role, Thread, ThreadStatus};
use crate::domain::error::DomainError;
use crate::domain::repository::{MessageRepository, ThreadRepository};
use crate::infra::http;
use chrono::Utc;
use uuid::Uuid;

pub struct ThreadUseCase<T: ThreadRepository, M: MessageRepository> {
    pub(crate) thread_repo: T,
    pub(crate) message_repo: M,
}

impl<T: ThreadRepository, M: MessageRepository> ThreadUseCase<T, M> {
    pub fn new(thread_repo: T, message_repo: M) -> Self {
        Self {
            thread_repo,
            message_repo,
        }
    }

    pub fn create(&self, title: &str) -> Result<Thread, DomainError> {
        let now = Utc::now();
        let thread = Thread {
            id: Uuid::new_v4().to_string(),
            name: None,
            title: title.to_string(),
            source_url: None,
            status: ThreadStatus::default(),
            created_at: now,
            updated_at: now,
        };
        self.thread_repo.create(&thread)?;
        Ok(thread)
    }

    pub fn list(&self) -> Result<Vec<Thread>, DomainError> {
        self.thread_repo.list()
    }

    pub fn list_by_status(&self, status: Option<ThreadStatus>) -> Result<Vec<Thread>, DomainError> {
        self.thread_repo.list_by_status(status)
    }

    pub fn find_by_id(&self, id: &str) -> Result<Option<Thread>, DomainError> {
        self.thread_repo.find_by_id(id)
    }

    pub fn resolve_id(&self, short_id: &str) -> Result<String, DomainError> {
        self.thread_repo.resolve_short_id(short_id)
    }

    pub fn close(&self, id: &str) -> Result<(), DomainError> {
        let full_id = self.thread_repo.resolve_short_id(id)?;
        self.thread_repo.update_status(&full_id, ThreadStatus::Closed)
    }

    pub fn reopen(&self, id: &str) -> Result<(), DomainError> {
        let full_id = self.thread_repo.resolve_short_id(id)?;
        self.thread_repo.update_status(&full_id, ThreadStatus::Open)
    }

    pub fn delete(&self, id: &str) -> Result<(), DomainError> {
        let full_id = self.thread_repo.resolve_short_id(id)?;
        self.message_repo.delete_by_thread(&full_id)?;
        self.thread_repo.delete(&full_id)
    }

    pub fn fetch(
        &self,
        url: &str,
        title: Option<&str>,
        sender: Option<&str>,
    ) -> Result<Thread, DomainError> {
        let html = http::fetch_url(url)?;
        let markdown = http::html_to_markdown(&html);

        let thread_title = title.unwrap_or(url);
        let now = Utc::now();
        let thread = Thread {
            id: Uuid::new_v4().to_string(),
            name: None,
            title: thread_title.to_string(),
            source_url: Some(url.to_string()),
            status: ThreadStatus::default(),
            created_at: now,
            updated_at: now,
        };
        self.thread_repo.create(&thread)?;

        let msg = Message {
            id: Uuid::new_v4().to_string(),
            thread_id: thread.id.clone(),
            session_id: None,
            sender: sender.map(|s| s.to_string()),
            role: Role::System,
            content: markdown,
            metadata: None,
            parent_id: None,
            source: Some("url-fetch".to_string()),
            created_at: now,
            updated_at: now,
        };
        self.message_repo.insert(&msg)?;

        Ok(thread)
    }
}
