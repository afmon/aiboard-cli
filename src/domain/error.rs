use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("thread not found: {0}")]
    ThreadNotFound(String),

    #[error("message not found: {0}")]
    MessageNotFound(String),

    #[error("ambiguous short id '{0}': matched {1} records")]
    AmbiguousShortId(String, usize),

    #[error("database error: {0}")]
    Database(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("I/O error: {0}")]
    Io(String),
}

impl DomainError {
    /// Returns the appropriate exit code for this error.
    /// 0 = success, 1 = general error, 2 = input error.
    pub fn exit_code(&self) -> i32 {
        match self {
            DomainError::InvalidInput(_) | DomainError::Parse(_) => 2,
            _ => 1,
        }
    }

    /// Returns true if this is an input validation error (exit code 2).
    pub fn is_input_error(&self) -> bool {
        self.exit_code() == 2
    }
}

impl From<std::io::Error> for DomainError {
    fn from(e: std::io::Error) -> Self {
        DomainError::Io(e.to_string())
    }
}

impl From<rusqlite::Error> for DomainError {
    fn from(e: rusqlite::Error) -> Self {
        DomainError::Database(e.to_string())
    }
}
