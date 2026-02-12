use std::io::Read;

use anyhow::{bail, Context};
use chrono::{DateTime, NaiveDateTime, Utc};

use crate::cli::args::*;
use crate::cli::formatter;
use crate::domain::entity::Role;
use crate::domain::repository::{MessageRepository, ThreadRepository};
use crate::usecase::cleanup::CleanupUseCase;
use crate::usecase::hook::HookUseCase;
use crate::usecase::message::MessageUseCase;
use crate::usecase::thread::ThreadUseCase;

const MAX_CONTENT_SIZE: usize = 1_048_576; // 1MB

fn read_stdin() -> anyhow::Result<String> {
    let mut buf = Vec::new();
    let bytes_read = std::io::stdin()
        .take(MAX_CONTENT_SIZE as u64 + 1)
        .read_to_end(&mut buf)
        .context("failed to read from stdin")?;

    if bytes_read > MAX_CONTENT_SIZE {
        bail!("input exceeds 1MB limit ({} bytes)", bytes_read);
    }

    if buf.iter().any(|&b| b == 0) {
        bail!("input contains NUL bytes");
    }

    String::from_utf8(buf).context("input is not valid UTF-8")
}

fn validate_content(content: &str) -> anyhow::Result<()> {
    if content.len() > MAX_CONTENT_SIZE {
        bail!("content exceeds 1MB limit ({} bytes)", content.len());
    }
    if content.bytes().any(|b| b == 0) {
        bail!("content contains NUL bytes");
    }
    Ok(())
}

fn parse_datetime_filter(s: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|ndt| ndt.and_utc())
}

pub fn handle_message<T: ThreadRepository, M: MessageRepository>(
    action: MessageAction,
    message_uc: &MessageUseCase<M>,
    _thread_uc: &ThreadUseCase<T, M>,
) -> anyhow::Result<()> {
    match action {
        MessageAction::Post {
            thread,
            role,
            content,
            session,
            sender,
            parent,
            metadata,
        } => {
            let body = match content {
                Some(c) => c,
                None => read_stdin()?,
            };
            validate_content(&body)?;

            let role: Role = role
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;

            let metadata_val: Option<serde_json::Value> = match metadata {
                Some(m) => {
                    let val: serde_json::Value = serde_json::from_str(&m)
                        .context("--metadata must be valid JSON")?;
                    Some(val)
                }
                None => None,
            };

            let msg = message_uc.post(
                &thread,
                role,
                &body,
                session.as_deref(),
                sender.as_deref(),
                metadata_val,
                parent.as_deref(),
            )?;
            println!("{}", formatter::format_message_posted(&msg));
        }

        MessageAction::Read {
            thread,
            limit,
            before,
            after,
            format,
        } => {
            let mut messages = message_uc.read(&thread)?;

            if let Some(dt) = after.as_deref().and_then(parse_datetime_filter) {
                messages.retain(|m| m.created_at > dt);
            }

            if let Some(dt) = before.as_deref().and_then(parse_datetime_filter) {
                messages.retain(|m| m.created_at < dt);
            }

            if let Some(lim) = limit {
                messages.truncate(lim);
            }

            match format.as_str() {
                "json" => println!("{}", formatter::format_messages_json(&messages)),
                _ => println!("{}", formatter::format_messages_text(&messages)),
            }
        }

        MessageAction::Search {
            query,
            thread,
            format,
        } => {
            let messages = message_uc.search(&query, thread.as_deref())?;
            match format.as_str() {
                "json" => println!("{}", formatter::format_messages_json(&messages)),
                _ => println!("{}", formatter::format_messages_text(&messages)),
            }
        }

        MessageAction::Update { id, content } => {
            validate_content(&content)?;
            let full_id = message_uc.update(&id, &content)?;
            println!("{}", full_id);
        }
    }
    Ok(())
}

pub fn handle_thread<T: ThreadRepository, M: MessageRepository>(
    action: ThreadAction,
    thread_uc: &ThreadUseCase<T, M>,
) -> anyhow::Result<()> {
    match action {
        ThreadAction::Create { title } => {
            let thread = thread_uc.create(&title)?;
            println!("{}", thread.id);
        }
        ThreadAction::List { format } => {
            let threads = thread_uc.list()?;
            match format.as_str() {
                "json" => println!("{}", formatter::format_threads_json(&threads)),
                _ => println!("{}", formatter::format_threads_text(&threads)),
            }
        }
        ThreadAction::Delete { id } => {
            thread_uc.delete(&id)?;
            eprintln!("deleted thread {}", id);
        }
        ThreadAction::Fetch { url, title, sender } => {
            eprintln!("fetching {}...", url);
            let thread = thread_uc.fetch(&url, title.as_deref(), sender.as_deref())?;
            println!("{}", thread.id);
            eprintln!("fetched and stored as thread {}", &thread.id[..8.min(thread.id.len())]);
        }
    }
    Ok(())
}

pub fn handle_hook<M: MessageRepository>(
    action: HookAction,
    hook_uc: &HookUseCase<M>,
) -> anyhow::Result<()> {
    match action {
        HookAction::Ingest { thread } => {
            let input = read_stdin()?;
            let count = hook_uc.ingest(&thread, &input)?;
            eprintln!("ingested {} messages", count);
        }
    }
    Ok(())
}

pub fn handle_cleanup<T: ThreadRepository, M: MessageRepository>(
    action: CleanupAction,
    cleanup_uc: &CleanupUseCase<T, M>,
) -> anyhow::Result<()> {
    match action {
        CleanupAction::Age { days } => {
            let count = cleanup_uc.by_age(days)?;
            eprintln!("deleted {} messages older than {} days", count, days);
        }
        CleanupAction::Thread { id } => {
            let count = cleanup_uc.by_thread(&id)?;
            eprintln!("deleted thread {} and {} messages", id, count);
        }
        CleanupAction::Session { id } => {
            let count = cleanup_uc.by_session(&id)?;
            eprintln!("deleted {} messages from session {}", count, id);
        }
    }
    Ok(())
}

pub fn handle_setup(action: SetupAction) -> anyhow::Result<()> {
    match action {
        SetupAction::Hooks { apply } => {
            let json_str = crate::usecase::setup::generate_hooks_string();

            if apply {
                let settings_path = std::path::Path::new(".claude").join("settings.json");

                eprint!(
                    "This will write hook configuration to {}. Continue? [y/N] ",
                    settings_path.display()
                );

                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .context("failed to read confirmation")?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    eprintln!("aborted");
                    return Ok(());
                }

                if let Some(parent) = settings_path.parent() {
                    std::fs::create_dir_all(parent)
                        .context("failed to create .claude directory")?;
                }

                // Merge into existing settings if present
                let mut settings = if settings_path.exists() {
                    let existing = std::fs::read_to_string(&settings_path)
                        .context("failed to read existing settings")?;
                    serde_json::from_str::<serde_json::Value>(&existing)
                        .unwrap_or_else(|_| serde_json::json!({}))
                } else {
                    serde_json::json!({})
                };

                let hooks_val = crate::usecase::setup::generate_hooks_json();
                if let Some(obj) = settings.as_object_mut() {
                    if let Some(hooks) = hooks_val.get("hooks") {
                        obj.insert("hooks".to_string(), hooks.clone());
                    }
                }

                let merged = serde_json::to_string_pretty(&settings)?;
                std::fs::write(&settings_path, &merged)
                    .context("failed to write settings file")?;

                eprintln!("wrote hook configuration to {}", settings_path.display());
            } else {
                println!("{}", json_str);
            }
        }

        SetupAction::Skill { apply } => {
            let content = crate::usecase::setup::generate_skill_content();

            if apply {
                let skill_dir = std::path::Path::new(".claude")
                    .join("skills")
                    .join("aiboard");
                let skill_path = skill_dir.join("SKILL.md");

                eprint!(
                    "This will write skill file to {}. Continue? [y/N] ",
                    skill_path.display()
                );

                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .context("failed to read confirmation")?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    eprintln!("aborted");
                    return Ok(());
                }

                std::fs::create_dir_all(&skill_dir)
                    .context("failed to create skills directory")?;
                std::fs::write(&skill_path, &content)
                    .context("failed to write skill file")?;

                eprintln!("wrote skill file to {}", skill_path.display());
            } else {
                println!("{}", content);
            }
        }
    }
    Ok(())
}
