use super::entity::{Message, Thread};
use super::error::DomainError;

pub trait ThreadRepository {
    fn create(&self, thread: &Thread) -> Result<(), DomainError>;
    fn upsert(&self, thread: &Thread) -> Result<(), DomainError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Thread>, DomainError>;
    fn resolve_short_id(&self, short_id: &str) -> Result<String, DomainError>;
    fn list(&self) -> Result<Vec<Thread>, DomainError>;
    fn delete(&self, id: &str) -> Result<(), DomainError>;
}

pub trait MessageRepository {
    fn insert(&self, message: &Message) -> Result<(), DomainError>;
    fn insert_batch(&self, messages: &[Message]) -> Result<usize, DomainError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Message>, DomainError>;
    fn resolve_short_id(&self, short_id: &str) -> Result<String, DomainError>;
    fn find_by_thread(&self, thread_id: &str) -> Result<Vec<Message>, DomainError>;
    fn list_recent(&self, limit: usize) -> Result<Vec<Message>, DomainError>;
    fn search(&self, query: &str, thread_id: Option<&str>) -> Result<Vec<Message>, DomainError>;
    fn update_content(&self, id: &str, content: &str) -> Result<(), DomainError>;
    fn delete_by_thread(&self, thread_id: &str) -> Result<usize, DomainError>;
    fn delete_by_session(&self, session_id: &str) -> Result<usize, DomainError>;
    fn delete_older_than(&self, before: &chrono::DateTime<chrono::Utc>) -> Result<usize, DomainError>;
}
