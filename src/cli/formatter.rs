use crate::domain::entity::{Message, Thread};

pub fn format_message_text(msg: &Message) -> String {
    let id_short = &msg.id[..8.min(msg.id.len())];
    let sender = msg.sender.as_deref().unwrap_or("-");
    format!(
        "[{}] {} ({}) {}: {}",
        msg.created_at.format("%Y-%m-%d %H:%M:%S"),
        id_short,
        msg.role,
        sender,
        msg.content,
    )
}

pub fn format_messages_text(messages: &[Message]) -> String {
    messages
        .iter()
        .map(format_message_text)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_messages_json(messages: &[Message]) -> String {
    serde_json::to_string_pretty(messages).unwrap_or_else(|_| "[]".to_string())
}

pub fn format_thread_text(thread: &Thread) -> String {
    let name = thread.name.as_deref().unwrap_or("-");
    format!(
        "{}\t{}\t{}\t{}",
        &thread.id[..8.min(thread.id.len())],
        name,
        thread.title,
        thread.updated_at.format("%Y-%m-%d %H:%M:%S"),
    )
}

pub fn format_threads_text(threads: &[Thread]) -> String {
    threads
        .iter()
        .map(format_thread_text)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_threads_json(threads: &[Thread]) -> String {
    serde_json::to_string_pretty(threads).unwrap_or_else(|_| "[]".to_string())
}

pub fn format_message_posted(msg: &Message) -> String {
    format!("{}", msg.id)
}
