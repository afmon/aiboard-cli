use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::domain::error::DomainError;

/// DB ファイルのバックアップを作成し、バックアップ先のパスを返す。
pub fn create_backup(db_path: &Path) -> Result<PathBuf, DomainError> {
    if !db_path.exists() {
        return Err(DomainError::Io(format!(
            "バックアップ対象のファイルが見つかりません: {}",
            db_path.display()
        )));
    }

    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let file_name = format!(
        "{}.bak.{}",
        db_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("aiboard.db"),
        timestamp
    );

    let backup_path = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(file_name);

    std::fs::copy(db_path, &backup_path).map_err(|e| {
        DomainError::Io(format!(
            "バックアップの作成に失敗しました: {}",
            e
        ))
    })?;

    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn create_backup_copies_file() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("aiboard.db");
        fs::write(&db_path, b"test data").unwrap();

        let backup_path = create_backup(&db_path).unwrap();

        assert!(backup_path.exists());
        assert_eq!(fs::read(&backup_path).unwrap(), b"test data");

        let name = backup_path.file_name().unwrap().to_str().unwrap();
        assert!(name.starts_with("aiboard.db.bak."));
    }

    #[test]
    fn create_backup_nonexistent_file_errors() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("nonexistent.db");

        let result = create_backup(&db_path);
        assert!(result.is_err());
    }
}
