use super::entity::{Message, Thread, ThreadPhase, ThreadStatus};
use super::error::DomainError;

pub trait ThreadRepository {
    fn create(&self, thread: &Thread) -> Result<(), DomainError>;
    fn upsert(&self, thread: &Thread) -> Result<(), DomainError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Thread>, DomainError>;
    fn resolve_short_id(&self, short_id: &str) -> Result<String, DomainError>;
    fn list(&self) -> Result<Vec<Thread>, DomainError>;
    fn list_by_status(&self, status: Option<ThreadStatus>) -> Result<Vec<Thread>, DomainError>;
    fn update_status(&self, id: &str, status: ThreadStatus) -> Result<(), DomainError>;
    fn update_phase(&self, id: &str, phase: Option<ThreadPhase>) -> Result<(), DomainError>;
    fn delete(&self, id: &str) -> Result<(), DomainError>;
}

pub trait MessageRepository {
    fn insert(&self, message: &Message) -> Result<(), DomainError>;
    fn insert_batch(&self, messages: &[Message]) -> Result<usize, DomainError>;
    #[allow(dead_code)]
    fn find_by_id(&self, id: &str) -> Result<Option<Message>, DomainError>;
    fn resolve_short_id(&self, short_id: &str) -> Result<String, DomainError>;
    fn find_by_thread(&self, thread_id: &str) -> Result<Vec<Message>, DomainError>;
    fn list_recent(&self, limit: usize) -> Result<Vec<Message>, DomainError>;
    fn search(&self, query: &str, thread_id: Option<&str>) -> Result<Vec<Message>, DomainError>;
    fn update_content(&self, id: &str, content: &str) -> Result<(), DomainError>;
    fn delete_by_thread(&self, thread_id: &str) -> Result<usize, DomainError>;
    fn delete_by_session(&self, session_id: &str) -> Result<usize, DomainError>;
    fn delete_older_than(&self, before: &chrono::DateTime<chrono::Utc>) -> Result<usize, DomainError>;
    fn find_mentions(&self, thread_id: Option<&str>, mention_target: &str) -> Result<Vec<Message>, DomainError>;
    fn count_mentions(&self, thread_id: Option<&str>, mention_target: &str) -> Result<usize, DomainError>;
    fn find_by_type(&self, thread_id: Option<&str>, msg_type: &str) -> Result<Vec<Message>, DomainError>;
    fn find_since_last_type(&self, thread_id: &str, msg_type: &str) -> Result<Vec<Message>, DomainError>;
}
