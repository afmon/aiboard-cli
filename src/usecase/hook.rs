use crate::domain::entity::{Message, Role, Thread, ThreadStatus};
use crate::domain::error::DomainError;
use crate::domain::repository::{MessageRepository, ThreadRepository};
use chrono::Utc;
use uuid::Uuid;

pub struct HookUseCase<T: ThreadRepository, R: MessageRepository> {
    pub(crate) thread_repo: T,
    pub(crate) repo: R,
}

impl<T: ThreadRepository, R: MessageRepository> HookUseCase<T, R> {
    pub fn new(thread_repo: T, repo: R) -> Self {
        Self { thread_repo, repo }
    }

    /// Ingest a Claude Code hook event from stdin JSON.
    ///
    /// The JSON contains common fields (session_id, hook_event_name, etc.)
    /// plus event-specific fields. A thread_id override can be provided
    /// via CLI; otherwise session_id is used as the thread_id.
    pub fn ingest(
        &self,
        thread_id_override: Option<&str>,
        json_input: &str,
    ) -> Result<usize, DomainError> {
        let parsed: serde_json::Value = serde_json::from_str(json_input)
            .map_err(|e| DomainError::Parse(format!("invalid JSON: {}", e)))?;

        let session_id = parsed
            .get("session_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let thread_id = match thread_id_override {
            Some(tid) => tid.to_string(),
            None => session_id
                .clone()
                .ok_or_else(|| DomainError::Parse("no session_id and no --thread provided".to_string()))?,
        };

        let event_name = parsed
            .get("hook_event_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let (role, content, sender, source) = match event_name {
            "UserPromptSubmit" => {
                let prompt = parsed
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                (Role::User, prompt, None, "user")
            }
            "PostToolUse" => {
                let tool_name = parsed
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if tool_name == "AskUserQuestion" {
                    match Self::parse_ask_user_question(&parsed) {
                        Some(content) => (Role::User, content, None, "user"),
                        None => return Ok(0),
                    }
                } else {
                    // Other tool events are skipped to avoid storing large outputs
                    return Ok(0);
                }
            }
            "Stop" => {
                let content = "[session stop]".to_string();
                (Role::Assistant, content, None, "system")
            }
            other => {
                let content = format!("[{}] event received", other);
                (Role::System, content, None, "system")
            }
        };

        if content.is_empty() {
            return Ok(0);
        }

        let now = Utc::now();

        // Ensure the thread exists (INSERT OR IGNORE)
        let short_id = &thread_id[..8.min(thread_id.len())];
        let thread = Thread {
            id: thread_id.clone(),
            name: None,
            title: format!("Session {}", short_id),
            source_url: None,
            status: ThreadStatus::default(),
            created_at: now,
            updated_at: now,
        };
        self.thread_repo.upsert(&thread)?;

        // クローズ済みスレッドへの投稿を警告
        if let Ok(Some(existing)) = self.thread_repo.find_by_id(&thread_id) {
            if existing.status == ThreadStatus::Closed {
                eprintln!("警告: thread {} はクローズされています", &thread_id[..8.min(thread_id.len())]);
            }
        }

        let message = Message {
            id: Uuid::new_v4().to_string(),
            thread_id,
            session_id,
            sender,
            role,
            content,
            metadata: None,
            parent_id: None,
            source: Some(source.to_string()),
            created_at: now,
            updated_at: now,
        };

        self.repo.insert_batch(&[message])
    }

    /// Parse AskUserQuestion tool_response into "Q: ... / A: ..." format.
    fn parse_ask_user_question(parsed: &serde_json::Value) -> Option<String> {
        let response = parsed.get("tool_response")?;

        // tool_response can be a JSON string or an object
        let obj = if let Some(s) = response.as_str() {
            serde_json::from_str::<serde_json::Value>(s).ok()?
        } else {
            response.clone()
        };

        let answers = obj.get("answers")?.as_object()?;
        if answers.is_empty() {
            return None;
        }

        let lines: Vec<String> = answers
            .iter()
            .map(|(q, a)| {
                let fallback = a.to_string();
                let answer = a.as_str().unwrap_or(&fallback);
                format!("Q: {} / A: {}", q, answer)
            })
            .collect();

        Some(format!("[決定] {}", lines.join(" | ")))
    }
}
