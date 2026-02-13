use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("thread が見つかりません: {0}")]
    ThreadNotFound(String),

    #[error("message が見つかりません: {0}")]
    MessageNotFound(String),

    #[error("短縮 ID '{0}' が曖昧です: {1} 件のレコードに一致")]
    AmbiguousShortId(String, usize),

    #[error("データベースエラー: {0}")]
    Database(String),

    #[error("入力が不正です: {0}")]
    InvalidInput(String),

    #[error("ネットワークエラー: {0}")]
    Network(String),

    #[error("パースエラー: {0}")]
    Parse(String),

    #[error("I/O エラー: {0}")]
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
