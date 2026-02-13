use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;

use crate::domain::entity::{Message, Role, Thread};
use crate::domain::error::DomainError;
use crate::domain::repository::{MessageRepository, ThreadRepository};

const MIGRATION_V1: &str = include_str!("migrations/v001.sql");
const MIGRATION_V2: &str = include_str!("migrations/v002.sql");


pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DomainError> {
        let conn = Connection::open(path)
            .map_err(|e| DomainError::Database(format!("failed to open database: {}", e)))?;

        Self::configure(&conn)?;
        let mut db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, DomainError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| DomainError::Database(format!("failed to open in-memory database: {}", e)))?;

        Self::configure(&conn)?;
        let mut db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn configure(conn: &Connection) -> Result<(), DomainError> {
        // foreign_keys = OFF: referential integrity is enforced at the application layer
        // (UseCase). This avoids FK-related performance overhead on bulk inserts and
        // keeps the schema compatible with FTS5 content-sync triggers.
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = OFF;"
        ).map_err(|e| DomainError::Database(format!("failed to configure database: {}", e)))
    }

    fn current_version(&self) -> Result<i64, DomainError> {
        let has_table: bool = self.conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| DomainError::Database(format!("failed to check schema_version table: {}", e)))?;

        if !has_table {
            return Ok(0);
        }

        let version: i64 = self.conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| DomainError::Database(format!("failed to read schema version: {}", e)))?;

        Ok(version)
    }

    fn migrate(&mut self) -> Result<(), DomainError> {
        let version = self.current_version()?;

        if version < 1 {
            self.conn
                .execute_batch(MIGRATION_V1)
                .map_err(|e| DomainError::Database(format!("migration v1 failed: {}", e)))?;
        }

        if version < 2 {
            self.conn
                .execute_batch(MIGRATION_V2)
                .map_err(|e| DomainError::Database(format!("migration v2 failed: {}", e)))?;
        }

        Ok(())
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

fn parse_datetime(s: &str) -> rusqlite::Result<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f"))
        .map(|ndt| ndt.and_utc())
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        ))
}

fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

// --- Thread Repository ---

pub struct SqliteThreadRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteThreadRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> ThreadRepository for SqliteThreadRepository<'a> {
    fn create(&self, thread: &Thread) -> Result<(), DomainError> {
        self.conn
            .execute(
                "INSERT INTO threads (id, name, title, source_url, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    thread.id,
                    thread.name,
                    thread.title,
                    thread.source_url,
                    format_datetime(&thread.created_at),
                    format_datetime(&thread.updated_at),
                ],
            )
            .map_err(|e| DomainError::Database(format!("failed to create thread: {}", e)))?;
        Ok(())
    }

    fn upsert(&self, thread: &Thread) -> Result<(), DomainError> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO threads (id, name, title, source_url, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    thread.id,
                    thread.name,
                    thread.title,
                    thread.source_url,
                    format_datetime(&thread.created_at),
                    format_datetime(&thread.updated_at),
                ],
            )
            .map_err(|e| DomainError::Database(format!("failed to upsert thread: {}", e)))?;
        Ok(())
    }

    fn resolve_short_id(&self, short_id: &str) -> Result<String, DomainError> {
        let pattern = format!("{}%", short_id);
        let mut stmt = self.conn
            .prepare("SELECT id FROM threads WHERE id LIKE ?1")?;

        let ids: Vec<String> = stmt
            .query_map(params![pattern], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        match ids.len() {
            0 => Err(DomainError::ThreadNotFound(short_id.to_string())),
            1 => Ok(ids.into_iter().next().unwrap()),
            n => Err(DomainError::AmbiguousShortId(short_id.to_string(), n)),
        }
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Thread>, DomainError> {
        let mut stmt = self.conn
            .prepare("SELECT id, name, title, source_url, created_at, updated_at FROM threads WHERE id = ?1")?;

        let result = stmt
            .query_row(params![id], |row| {
                Ok(Thread {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    title: row.get(2)?,
                    source_url: row.get(3)?,
                    created_at: parse_datetime(&row.get::<_, String>(4)?)?,
                    updated_at: parse_datetime(&row.get::<_, String>(5)?)?,
                })
            });

        match result {
            Ok(thread) => Ok(Some(thread)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list(&self) -> Result<Vec<Thread>, DomainError> {
        let mut stmt = self.conn
            .prepare("SELECT id, name, title, source_url, created_at, updated_at FROM threads ORDER BY updated_at DESC")?;

        let threads = stmt
            .query_map([], |row| {
                Ok(Thread {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    title: row.get(2)?,
                    source_url: row.get(3)?,
                    created_at: parse_datetime(&row.get::<_, String>(4)?)?,
                    updated_at: parse_datetime(&row.get::<_, String>(5)?)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(threads)
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let affected = self.conn
            .execute("DELETE FROM threads WHERE id = ?1", params![id])?;

        if affected == 0 {
            return Err(DomainError::ThreadNotFound(id.to_string()));
        }
        Ok(())
    }
}

// --- Message Repository ---

pub struct SqliteMessageRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteMessageRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<Message> {
        let role_str: String = row.get(4)?;
        let role = role_str.parse::<Role>().unwrap_or(Role::User);
        let metadata_str: Option<String> = row.get(6)?;
        let metadata = metadata_str
            .and_then(|s| serde_json::from_str(&s).ok());

        Ok(Message {
            id: row.get(0)?,
            thread_id: row.get(1)?,
            session_id: row.get(2)?,
            sender: row.get(3)?,
            role,
            content: row.get(5)?,
            metadata,
            parent_id: row.get(7)?,
            source: row.get(8)?,
            created_at: parse_datetime(&row.get::<_, String>(9)?)?,
            updated_at: parse_datetime(&row.get::<_, String>(10)?)?,
        })
    }
}

impl<'a> MessageRepository for SqliteMessageRepository<'a> {
    fn insert(&self, message: &Message) -> Result<(), DomainError> {
        let metadata_json = message
            .metadata
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()));

        self.conn
            .execute(
                "INSERT INTO messages (id, thread_id, session_id, sender, role, content, metadata, parent_id, source, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    message.id,
                    message.thread_id,
                    message.session_id,
                    message.sender,
                    message.role.to_string(),
                    message.content,
                    metadata_json,
                    message.parent_id,
                    message.source,
                    format_datetime(&message.created_at),
                    format_datetime(&message.updated_at),
                ],
            )
            .map_err(|e| DomainError::Database(format!("failed to insert message: {}", e)))?;
        Ok(())
    }

    fn insert_batch(&self, messages: &[Message]) -> Result<usize, DomainError> {
        self.conn
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| DomainError::Database(format!("failed to begin transaction: {}", e)))?;

        let result = messages.iter().try_for_each(|msg| self.insert(msg));

        match result {
            Ok(()) => {
                self.conn
                    .execute_batch("COMMIT")
                    .map_err(|e| DomainError::Database(format!("failed to commit transaction: {}", e)))?;
                Ok(messages.len())
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Message>, DomainError> {
        let mut stmt = self.conn
            .prepare(
                "SELECT id, thread_id, session_id, sender, role, content, metadata, parent_id, source, created_at, updated_at
                 FROM messages WHERE id = ?1"
            )?;

        let result = stmt.query_row(params![id], Self::row_to_message);

        match result {
            Ok(msg) => Ok(Some(msg)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn resolve_short_id(&self, short_id: &str) -> Result<String, DomainError> {
        let pattern = format!("{}%", short_id);
        let mut stmt = self.conn
            .prepare("SELECT id FROM messages WHERE id LIKE ?1")?;

        let ids: Vec<String> = stmt
            .query_map(params![pattern], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        match ids.len() {
            0 => Err(DomainError::MessageNotFound(short_id.to_string())),
            1 => Ok(ids.into_iter().next().unwrap()),
            n => Err(DomainError::AmbiguousShortId(short_id.to_string(), n)),
        }
    }

    fn find_by_thread(&self, thread_id: &str) -> Result<Vec<Message>, DomainError> {
        let mut stmt = self.conn
            .prepare(
                "SELECT id, thread_id, session_id, sender, role, content, metadata, parent_id, source, created_at, updated_at
                 FROM messages WHERE thread_id = ?1 ORDER BY created_at ASC"
            )?;

        let messages = stmt
            .query_map(params![thread_id], Self::row_to_message)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    fn list_recent(&self, limit: usize) -> Result<Vec<Message>, DomainError> {
        let mut stmt = self.conn
            .prepare(
                "SELECT id, thread_id, session_id, sender, role, content, metadata, parent_id, source, created_at, updated_at
                 FROM messages ORDER BY created_at DESC LIMIT ?1"
            )?;

        let messages = stmt
            .query_map(params![limit], Self::row_to_message)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    fn search(&self, query: &str, thread_id: Option<&str>) -> Result<Vec<Message>, DomainError> {
        // Try FTS5 first, fall back to LIKE
        self.search_fts(query, thread_id)
            .or_else(|_| self.search_like(query, thread_id))
    }

    fn update_content(&self, id: &str, content: &str) -> Result<(), DomainError> {
        let now = format_datetime(&Utc::now());
        let affected = self.conn
            .execute(
                "UPDATE messages SET content = ?1, updated_at = ?2 WHERE id = ?3",
                params![content, now, id],
            )?;

        if affected == 0 {
            return Err(DomainError::MessageNotFound(id.to_string()));
        }
        Ok(())
    }

    fn delete_by_thread(&self, thread_id: &str) -> Result<usize, DomainError> {
        Ok(self.conn
            .execute("DELETE FROM messages WHERE thread_id = ?1", params![thread_id])?)
    }

    fn delete_by_session(&self, session_id: &str) -> Result<usize, DomainError> {
        Ok(self.conn
            .execute("DELETE FROM messages WHERE session_id = ?1", params![session_id])?)
    }

    fn delete_older_than(&self, before: &DateTime<Utc>) -> Result<usize, DomainError> {
        let cutoff = format_datetime(before);
        Ok(self.conn
            .execute("DELETE FROM messages WHERE created_at < ?1", params![cutoff])?)
    }

    fn find_mentions(&self, thread_id: Option<&str>, mention_target: &str) -> Result<Vec<Message>, DomainError> {
        let escaped = mention_target.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%@{}%", escaped);

        let messages: Vec<Message> = match thread_id {
            Some(tid) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, thread_id, session_id, sender, role, content, metadata, parent_id, source, created_at, updated_at
                     FROM messages WHERE thread_id = ?1 AND content LIKE ?2 ESCAPE '\\' ORDER BY created_at DESC"
                )?;
                let rows = stmt.query_map(params![tid, pattern], Self::row_to_message)?
                    .collect::<Result<Vec<_>, _>>()?;
                rows
            }
            None => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, thread_id, session_id, sender, role, content, metadata, parent_id, source, created_at, updated_at
                     FROM messages WHERE content LIKE ?1 ESCAPE '\\' ORDER BY created_at DESC"
                )?;
                let rows = stmt.query_map(params![pattern], Self::row_to_message)?
                    .collect::<Result<Vec<_>, _>>()?;
                rows
            }
        };

        Ok(Self::filter_mention_boundary(messages, mention_target))
    }

    fn count_mentions(&self, thread_id: Option<&str>, mention_target: &str) -> Result<usize, DomainError> {
        self.find_mentions(thread_id, mention_target).map(|v| v.len())
    }
}

impl<'a> SqliteMessageRepository<'a> {
    /// Filter messages to ensure `@mention_target` is followed by a non-word character or EOF.
    /// This prevents `@alice` from matching `@alicex`.
    fn filter_mention_boundary(messages: Vec<Message>, mention_target: &str) -> Vec<Message> {
        let mention = format!("@{}", mention_target);
        messages.into_iter().filter(|msg| {
            let content = &msg.content;
            let mut start = 0;
            while let Some(pos) = content[start..].find(&mention) {
                let abs_pos = start + pos + mention.len();
                if abs_pos >= content.len() {
                    return true; // mention at EOF
                }
                let next_char = content[abs_pos..].chars().next().unwrap();
                if !next_char.is_alphanumeric() && next_char != '_' {
                    return true; // followed by non-word character
                }
                start = start + pos + 1;
            }
            false
        }).collect()
    }

    fn query_messages(&self, base_sql: &str, thread_filter: &str, search_param: &str, thread_id: Option<&str>) -> Result<Vec<Message>, DomainError> {
        let sql = match thread_id {
            Some(_) => format!("{} {} ORDER BY created_at DESC", base_sql, thread_filter),
            None => format!("{} ORDER BY created_at DESC", base_sql),
        };

        let mut stmt = self.conn.prepare(&sql)?;

        let messages: Vec<Message> = match thread_id {
            Some(tid) => {
                stmt.query_map(params![search_param, tid], Self::row_to_message)?
                    .collect::<Result<Vec<_>, _>>()?
            }
            None => {
                stmt.query_map(params![search_param], Self::row_to_message)?
                    .collect::<Result<Vec<_>, _>>()?
            }
        };

        Ok(messages)
    }

    fn search_fts(&self, query: &str, thread_id: Option<&str>) -> Result<Vec<Message>, DomainError> {
        self.query_messages(
            "SELECT m.id, m.thread_id, m.session_id, m.sender, m.role, m.content, m.metadata, m.parent_id, m.source, m.created_at, m.updated_at
             FROM messages m
             JOIN messages_fts fts ON m.rowid = fts.rowid
             WHERE messages_fts MATCH ?1",
            "AND m.thread_id = ?2",
            query,
            thread_id,
        )
    }

    fn search_like(&self, query: &str, thread_id: Option<&str>) -> Result<Vec<Message>, DomainError> {
        let escaped = query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%{}%", escaped);
        self.query_messages(
            "SELECT id, thread_id, session_id, sender, role, content, metadata, parent_id, source, created_at, updated_at
             FROM messages WHERE content LIKE ?1 ESCAPE '\\'",
            "AND thread_id = ?2",
            &pattern,
            thread_id,
        )
    }
}
