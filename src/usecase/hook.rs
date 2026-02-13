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
                // Extract main agent's last response from transcript_path
                match Self::parse_transcript_last_assistant(&parsed, "transcript_path") {
                    Some(content) => {
                        (Role::Assistant, content, Some("claude".to_string()), "agent")
                    }
                    None => return Ok(0),
                }
            }
            "SubagentStop" => {
                let agent_type = parsed
                    .get("agent_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                match Self::parse_transcript_last_assistant(&parsed, "agent_transcript_path") {
                    Some(content) => {
                        let sender = format!("subagent:{}", agent_type);
                        (Role::Assistant, content, Some(sender), "agent")
                    }
                    None => {
                        // Fallback if transcript is unavailable
                        let content = "[SubagentStop] event received".to_string();
                        (Role::System, content, None, "system")
                    }
                }
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
            phase: None,
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

    /// Extract the last assistant message from a transcript JSONL file.
    /// `path_key` specifies which JSON field contains the transcript path
    /// ("transcript_path" for Stop, "agent_transcript_path" for SubagentStop).
    fn parse_transcript_last_assistant(
        parsed: &serde_json::Value,
        path_key: &str,
    ) -> Option<String> {
        let transcript_path = parsed
            .get(path_key)
            .and_then(|v| v.as_str())?;

        // Read the transcript file (JSONL format)
        let content = std::fs::read_to_string(transcript_path).ok()?;

        // Parse JSONL and find the last assistant message
        let mut last_assistant_content: Option<String> = None;

        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(role) = entry.get("role").and_then(|r| r.as_str()) {
                    if role == "assistant" {
                        if let Some(text) = Self::extract_text_content(&entry) {
                            last_assistant_content = Some(text);
                        }
                    }
                }
            }
        }

        last_assistant_content
    }

    /// Extract text from a transcript entry's "content" field.
    /// Handles both string format and array-of-blocks format:
    ///   - String: "content": "hello"
    ///   - Array:  "content": [{"type":"text","text":"hello"}, ...]
    fn extract_text_content(entry: &serde_json::Value) -> Option<String> {
        let content = entry.get("content")?;

        // Case 1: content is a plain string
        if let Some(s) = content.as_str() {
            if s.is_empty() {
                return None;
            }
            return Some(s.to_string());
        }

        // Case 2: content is an array of content blocks
        if let Some(arr) = content.as_array() {
            let texts: Vec<&str> = arr
                .iter()
                .filter_map(|block| {
                    if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                        block.get("text").and_then(|t| t.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            if texts.is_empty() {
                return None;
            }
            return Some(texts.join("\n"));
        }

        None
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
