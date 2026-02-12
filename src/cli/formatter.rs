use crate::domain::entity::{Message, Thread};
use chrono::Local;

const TRUNCATE_LEN: usize = 100;
const SNIPPET_CONTEXT: usize = 50;

fn truncate_content(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max).collect();
    format!("{}…", truncated)
}

fn snippet_around(s: &str, query: &str, context: usize) -> String {
    let lower = s.to_lowercase();
    let q_lower = query.to_lowercase();
    let Some(byte_pos) = lower.find(&q_lower) else {
        return truncate_content(s, TRUNCATE_LEN);
    };

    let chars: Vec<char> = s.chars().collect();
    let total = chars.len();

    // Convert byte position to char position
    let char_pos = s[..byte_pos].chars().count();
    let q_chars = query.chars().count();

    let start = char_pos.saturating_sub(context);
    let end = (char_pos + q_chars + context).min(total);

    let slice: String = chars[start..end].iter().collect();

    let prefix = if start > 0 { "…" } else { "" };
    let suffix = if end < total { "…" } else { "" };

    format!("{}{}{}", prefix, slice, suffix)
}

fn format_message_with_content(msg: &Message, content: &str) -> String {
    let id_short = &msg.id[..8.min(msg.id.len())];
    let sender = msg.sender.as_deref().unwrap_or("-");
    let source_tag = match msg.source.as_deref() {
        Some(s) => format!(" [{}]", s),
        None => String::new(),
    };
    let local_time = msg.created_at.with_timezone(&Local);
    format!(
        "[{}] {} ({}) {}{}: {}",
        local_time.format("%Y-%m-%d %H:%M:%S"),
        id_short,
        msg.role,
        sender,
        source_tag,
        content,
    )
}

pub fn format_message_text(msg: &Message) -> String {
    format_message_with_content(msg, &msg.content)
}

pub fn format_message_truncated(msg: &Message) -> String {
    let content = truncate_content(&msg.content, TRUNCATE_LEN);
    format_message_with_content(msg, &content)
}

pub fn format_message_snippet(msg: &Message, query: &str) -> String {
    let content = snippet_around(&msg.content, query, SNIPPET_CONTEXT);
    format_message_with_content(msg, &content)
}

pub fn format_messages_text(messages: &[Message], full: bool) -> String {
    let fmt = if full { format_message_text } else { format_message_truncated };
    messages
        .iter()
        .map(fmt)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_messages_search(messages: &[Message], query: &str, full: bool) -> String {
    messages
        .iter()
        .map(|msg| {
            if full {
                format_message_text(msg)
            } else {
                format_message_snippet(msg, query)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_messages_json(messages: &[Message]) -> String {
    serde_json::to_string_pretty(messages).unwrap_or_else(|_| "[]".to_string())
}

pub fn format_thread_text(thread: &Thread, full: bool) -> String {
    let name = thread.name.as_deref().unwrap_or("-");
    let id = if full {
        &thread.id
    } else {
        &thread.id[..8.min(thread.id.len())]
    };
    let local_time = thread.updated_at.with_timezone(&Local);
    format!(
        "{}\t{}\t{}\t{}",
        id,
        name,
        thread.title,
        local_time.format("%Y-%m-%d %H:%M:%S"),
    )
}

pub fn format_threads_text(threads: &[Thread], full: bool) -> String {
    threads
        .iter()
        .map(|t| format_thread_text(t, full))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_threads_json(threads: &[Thread]) -> String {
    serde_json::to_string_pretty(threads).unwrap_or_else(|_| "[]".to_string())
}

pub fn format_message_posted(msg: &Message) -> String {
    format!("{}", msg.id)
}
