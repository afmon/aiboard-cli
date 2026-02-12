use crate::domain::error::DomainError;
use crate::domain::repository::{MessageRepository, ThreadRepository};
use chrono::{Duration, Utc};

pub struct CleanupUseCase<T: ThreadRepository, M: MessageRepository> {
    pub(crate) thread_repo: T,
    pub(crate) message_repo: M,
}

impl<T: ThreadRepository, M: MessageRepository> CleanupUseCase<T, M> {
    pub fn new(thread_repo: T, message_repo: M) -> Self {
        Self {
            thread_repo,
            message_repo,
        }
    }

    pub fn by_age(&self, days: i64) -> Result<usize, DomainError> {
        let cutoff = Utc::now() - Duration::days(days);
        self.message_repo.delete_older_than(&cutoff)
    }

    pub fn by_thread(&self, thread_id: &str) -> Result<usize, DomainError> {
        let count = self.message_repo.delete_by_thread(thread_id)?;
        self.thread_repo.delete(thread_id)?;
        Ok(count)
    }

    pub fn by_session(&self, session_id: &str) -> Result<usize, DomainError> {
        self.message_repo.delete_by_session(session_id)
    }
}
