use serde_json::json;

/// Generates the Claude Code hooks configuration JSON for aiboard integration.
/// This hooks into the PostToolUse event to capture conversation data.
pub fn generate_hooks_json() -> serde_json::Value {
    json!({
        "hooks": {
            "PostToolUse": [
                {
                    "matcher": ".*",
                    "command": "aiboard hook ingest --thread \"$THREAD_ID\""
                }
            ]
        }
    })
}

/// Returns the hooks configuration as a formatted JSON string.
pub fn generate_hooks_string() -> String {
    serde_json::to_string_pretty(&generate_hooks_json()).unwrap()
}

/// Generates the aiboard skill SKILL.md content for Claude Code integration.
pub fn generate_skill_content() -> String {
    r#"---
name: aiboard
description: Inter-agent communication and conversation log persistence via aiboard CLI
---

# aiboard Skill

Use the `aiboard` CLI tool to manage inter-agent communication and conversation logs.

## Commands

### Message Management
- `aiboard message post --thread <id> --content <text>` - Post a message
- `aiboard message read --thread <id>` - Read messages from a thread
- `aiboard message search <query>` - Search messages
- `aiboard message update <id> --content <text>` - Update a message

### Thread Management
- `aiboard thread create <title>` - Create a new thread
- `aiboard thread list` - List all threads
- `aiboard thread delete <id>` - Delete a thread
- `aiboard thread fetch <url>` - Fetch and store a conversation from a URL

### Cleanup
- `aiboard cleanup age <days>` - Delete old messages
- `aiboard cleanup thread <id>` - Delete a thread and its messages
- `aiboard cleanup session <id>` - Delete all messages from a session
"#
    .to_string()
}
