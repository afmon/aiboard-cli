use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aiboard", about = "Inter-agent communication and conversation log persistence")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage messages (post, read, search, update)
    Message {
        #[command(subcommand)]
        action: MessageAction,
    },
    /// Manage threads (create, list, delete, fetch)
    Thread {
        #[command(subcommand)]
        action: ThreadAction,
    },
    /// Ingest conversation logs from hooks
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
    /// Clean up old data
    Cleanup {
        #[command(subcommand)]
        action: CleanupAction,
    },
    /// Configure hooks and skills
    Setup {
        #[command(subcommand)]
        action: SetupAction,
    },
}

#[derive(Subcommand)]
pub enum MessageAction {
    /// Post a new message to a thread
    Post {
        /// Thread ID
        #[arg(long)]
        thread: String,
        /// Message role (user, assistant, system, tool)
        #[arg(long, default_value = "user")]
        role: String,
        /// Message content (reads from stdin if omitted)
        #[arg(long)]
        content: Option<String>,
        /// Session ID
        #[arg(long)]
        session: Option<String>,
        /// Sender name
        #[arg(long)]
        sender: Option<String>,
        /// Parent message ID
        #[arg(long)]
        parent: Option<String>,
        /// Metadata as JSON string
        #[arg(long)]
        metadata: Option<String>,
    },
    /// Read messages from a thread
    Read {
        /// Thread ID
        #[arg(long)]
        thread: String,
        /// Maximum number of messages to return
        #[arg(long)]
        limit: Option<usize>,
        /// Only messages before this datetime (ISO 8601)
        #[arg(long)]
        before: Option<String>,
        /// Only messages after this datetime (ISO 8601)
        #[arg(long)]
        after: Option<String>,
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Search messages
    Search {
        /// Search query
        query: String,
        /// Limit search to a specific thread
        #[arg(long)]
        thread: Option<String>,
        /// Output format (text, json, markdown)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Update a message's content
    Update {
        /// Message ID (short prefix allowed)
        id: String,
        /// New content
        #[arg(long)]
        content: String,
    },
}

#[derive(Subcommand)]
pub enum ThreadAction {
    /// Create a new thread
    Create {
        /// Thread title
        title: String,
    },
    /// List all threads
    List {
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Delete a thread and its messages
    Delete {
        /// Thread ID
        id: String,
    },
    /// Fetch a conversation from a URL and store it
    Fetch {
        /// Source URL
        url: String,
        /// Thread title (defaults to URL)
        #[arg(long)]
        title: Option<String>,
        /// Sender name for the fetched content
        #[arg(long)]
        sender: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum HookAction {
    /// Ingest conversation JSON from stdin
    Ingest {
        /// Thread ID to store messages in
        #[arg(long)]
        thread: String,
    },
}

#[derive(Subcommand)]
pub enum CleanupAction {
    /// Delete messages older than N days
    Age {
        /// Number of days
        days: i64,
    },
    /// Delete a thread and all its messages
    Thread {
        /// Thread ID
        id: String,
    },
    /// Delete all messages from a session
    Session {
        /// Session ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum SetupAction {
    /// Generate hook configuration for Claude Code
    Hooks {
        /// Apply the generated configuration to .claude/settings.json
        #[arg(long)]
        apply: bool,
    },
    /// Generate aiboard skill file for Claude Code
    Skill {
        /// Apply the generated skill to .claude/skills/
        #[arg(long)]
        apply: bool,
    },
}
