use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("aiboard").unwrap()
}

/// Test helper: create a temp dir and return its path as a String.
fn test_db() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap().to_string();
    (dir, path)
}

/// Test helper: create a thread and return its ID.
fn create_thread(db_path: &str, title: &str) -> String {
    let output = cmd()
        .args(["thread", "create", title])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    assert!(output.status.success(), "failed to create thread '{}'", title);
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

/// Test helper: post a message and return its ID.
fn post_message(db_path: &str, thread_id: &str, content: &str) -> String {
    let output = cmd()
        .args(["message", "post", "--thread", thread_id, "--content", content])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    assert!(output.status.success(), "failed to post message");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn help_prints_usage() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("エージェント間通信"));
}

#[test]
fn message_help_prints_subcommands() {
    cmd()
        .args(["message", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("post"))
        .stdout(predicate::str::contains("read"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("update"));
}

#[test]
fn thread_help_prints_subcommands() {
    cmd()
        .args(["thread", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("fetch"));
}

#[test]
fn thread_create_list_delete() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // Create a thread
    let output = cmd()
        .args(["thread", "create", "test-thread"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();
    assert!(!thread_id.is_empty());

    // List threads
    cmd()
        .args(["thread", "list"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("test-thread"));

    // List threads as JSON
    cmd()
        .args(["thread", "list", "--format", "json"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\""))
        .stdout(predicate::str::contains("test-thread"));

    // Delete thread
    cmd()
        .args(["thread", "delete", &thread_id])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    // List should be empty now
    let output = cmd()
        .args(["thread", "list"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.trim().is_empty());
}

#[test]
fn message_post_read() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // Create a thread
    let output = cmd()
        .args(["thread", "create", "msg-test"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Post a message with --content
    let output = cmd()
        .args([
            "message", "post",
            "--thread", &thread_id,
            "--role", "user",
            "--content", "Hello, world!",
            "--sender", "test-agent",
        ])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let msg_id = String::from_utf8(output.stdout).unwrap().trim().to_string();
    assert!(!msg_id.is_empty());

    // Read messages
    cmd()
        .args(["message", "read", "--thread", &thread_id])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, world!"))
        .stdout(predicate::str::contains("test-agent"));

    // Read messages as JSON
    cmd()
        .args(["message", "read", "--thread", &thread_id, "--format", "json"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\""))
        .stdout(predicate::str::contains("Hello, world!"));
}

#[test]
fn message_post_from_stdin() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // Create a thread
    let output = cmd()
        .args(["thread", "create", "stdin-test"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Post via stdin
    cmd()
        .args(["message", "post", "--thread", &thread_id])
        .write_stdin("message from stdin")
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    // Read and verify
    cmd()
        .args(["message", "read", "--thread", &thread_id])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("message from stdin"));
}

#[test]
fn message_search() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // Create thread and post messages
    let output = cmd()
        .args(["thread", "create", "search-test"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    cmd()
        .args(["message", "post", "--thread", &thread_id, "--content", "the quick brown fox"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    cmd()
        .args(["message", "post", "--thread", &thread_id, "--content", "lazy dog sleeps"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    // Search for "fox"
    cmd()
        .args(["message", "search", "fox"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("quick brown fox"));

    // Search for "dog"
    cmd()
        .args(["message", "search", "dog"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("lazy dog"));
}

#[test]
fn message_update() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let output = cmd()
        .args(["thread", "create", "update-test"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    let output = cmd()
        .args(["message", "post", "--thread", &thread_id, "--content", "original content"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let msg_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Update using short ID (first 8 chars)
    let short_id = &msg_id[..8];
    cmd()
        .args(["message", "update", short_id, "--content", "updated content"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(&msg_id));

    // Verify update
    cmd()
        .args(["message", "read", "--thread", &thread_id])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("updated content"));
}

#[test]
fn cleanup_by_session() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let output = cmd()
        .args(["thread", "create", "cleanup-test"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Post with session ID
    cmd()
        .args([
            "message", "post",
            "--thread", &thread_id,
            "--content", "session message",
            "--session", "sess-123",
        ])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    // Delete by session
    cmd()
        .args(["cleanup", "session", "sess-123"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    // Messages should be gone
    let output = cmd()
        .args(["message", "read", "--thread", &thread_id])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("session message"));
}

#[test]
fn hook_ingest() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let thread_id = create_thread(db_path, "hook-test");

    // Ingest a UserPromptSubmit event
    let json = serde_json::json!({
        "session_id": "hook-session-1",
        "hook_event_name": "UserPromptSubmit",
        "prompt": "hello from hook"
    });

    cmd()
        .args(["hook", "ingest", "--thread", &thread_id])
        .write_stdin(json.to_string())
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    // Verify
    cmd()
        .args(["message", "read", "--thread", &thread_id])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello from hook"));
}

#[test]
fn invalid_role_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let output = cmd()
        .args(["thread", "create", "invalid-role"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    cmd()
        .args([
            "message", "post",
            "--thread", &thread_id,
            "--role", "invalid_role",
            "--content", "test",
        ])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .failure();
}

#[test]
fn no_subcommand_shows_help() {
    cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn cleanup_age_zero() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let output = cmd()
        .args(["thread", "create", "age-cleanup-test"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    cmd()
        .args(["message", "post", "--thread", &thread_id, "--content", "old message"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    // cleanup age 0 should delete all messages (everything is older than 0 days from now)
    cmd()
        .args(["cleanup", "age", "0"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();
}

#[test]
fn invalid_metadata_json_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let output = cmd()
        .args(["thread", "create", "meta-test"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    cmd()
        .args([
            "message", "post",
            "--thread", &thread_id,
            "--content", "test",
            "--metadata", "not valid json{{{",
        ])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .failure();
}

#[test]
fn nonexistent_thread_read() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // Reading from a nonexistent thread should fail with ThreadNotFound
    cmd()
        .args(["message", "read", "--thread", "nonexistent-thread-id"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .failure();
}

#[test]
fn nonexistent_thread_delete() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // Deleting a nonexistent thread should fail
    cmd()
        .args(["thread", "delete", "nonexistent-thread-id"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .failure();
}

#[test]
fn thread_list_json_is_valid_json() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    cmd()
        .args(["thread", "create", "json-test-1"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    cmd()
        .args(["thread", "create", "json-test-2"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    let output = cmd()
        .args(["thread", "list", "--format", "json"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("thread list --format json should output valid JSON");
    assert!(parsed.is_array());
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

#[test]
fn message_read_json_is_valid_json() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let output = cmd()
        .args(["thread", "create", "json-msg-test"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    cmd()
        .args(["message", "post", "--thread", &thread_id, "--content", "json test message"])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    let output = cmd()
        .args(["message", "read", "--thread", &thread_id, "--format", "json"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("message read --format json should output valid JSON");
    assert!(parsed.is_array());
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["content"], "json test message");
}

#[test]
fn message_post_with_valid_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let output = cmd()
        .args(["thread", "create", "valid-meta-test"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let thread_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    cmd()
        .args([
            "message", "post",
            "--thread", &thread_id,
            "--content", "with metadata",
            "--metadata", r#"{"key": "value", "num": 42}"#,
        ])
        .env("AIBOARD_DATA_DIR", db_path)
        .assert()
        .success();

    // Verify metadata is stored by reading as JSON
    let output = cmd()
        .args(["message", "read", "--thread", &thread_id, "--format", "json"])
        .env("AIBOARD_DATA_DIR", db_path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let msg = &parsed.as_array().unwrap()[0];
    assert_eq!(msg["metadata"]["key"], "value");
    assert_eq!(msg["metadata"]["num"], 42);
}

#[test]
fn setup_hooks_generates_json() {
    let output = cmd()
        .args(["setup", "hooks"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("setup hooks should output valid JSON");

    let hooks = parsed.get("hooks").expect("should have 'hooks' key");

    // All three events must be present
    for event in &["UserPromptSubmit", "PostToolUse", "Stop"] {
        let event_arr = hooks.get(*event)
            .unwrap_or_else(|| panic!("missing event: {}", event))
            .as_array()
            .unwrap_or_else(|| panic!("{} should be an array", event));
        assert!(!event_arr.is_empty(), "{} should have at least one entry", event);

        let entry = &event_arr[0];
        assert!(entry.get("matcher").is_some(), "{} entry should have 'matcher'", event);

        let inner_hooks = entry.get("hooks")
            .unwrap_or_else(|| panic!("{} entry should have 'hooks' array", event))
            .as_array()
            .unwrap_or_else(|| panic!("{} 'hooks' should be an array", event));
        assert!(!inner_hooks.is_empty());

        let hook_def = &inner_hooks[0];
        assert_eq!(hook_def.get("type").and_then(|v| v.as_str()), Some("command"),
            "{} hook type should be 'command'", event);
        assert_eq!(hook_def.get("async").and_then(|v| v.as_bool()), Some(true),
            "{} hook should be async", event);

        let command = hook_def.get("command").and_then(|v| v.as_str()).unwrap();
        assert!(command.contains("aiboard hook ingest"),
            "{} command should contain 'aiboard hook ingest'", event);
        assert!(!command.contains("--thread"),
            "{} command should not contain '--thread'", event);
    }
}

#[test]
fn setup_skill_generates_markdown() {
    cmd()
        .args(["setup", "skill"])
        .assert()
        .success()
        .stdout(predicate::str::contains("aiboard"))
        .stdout(predicate::str::contains("message post"));
}

// --- Security edge case tests ---

#[test]
fn nul_byte_in_content_rejected() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "nul-test");

    cmd()
        .args(["message", "post", "--thread", &thread_id])
        .write_stdin("hello\0world")
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("NUL"));
}

#[test]
fn fts5_special_chars_handled() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "fts5-special");

    post_message(&db_path, &thread_id, "normal content here");

    // Search with FTS5 special characters should not crash
    cmd()
        .args(["message", "search", "content*"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    // Quotes and parentheses (FTS5 syntax)
    cmd()
        .args(["message", "search", r#""quoted phrase""#])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();
}

#[test]
fn search_with_sql_wildcards() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "wildcard-test");

    post_message(&db_path, &thread_id, "100% complete");
    post_message(&db_path, &thread_id, "file_name.txt");

    // Search for literal % - should find the message
    cmd()
        .args(["message", "search", "100%", "--thread", &thread_id])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("100% complete"));

    // Search for literal _ - should find the message
    cmd()
        .args(["message", "search", "file_name", "--thread", &thread_id])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("file_name.txt"));
}

// --- CLI filter tests ---

#[test]
fn message_read_with_limit() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "limit-test");

    post_message(&db_path, &thread_id, "message one");
    post_message(&db_path, &thread_id, "message two");
    post_message(&db_path, &thread_id, "message three");

    // Limit to 2 messages
    let output = cmd()
        .args(["message", "read", "--thread", &thread_id, "--limit", "2", "--format", "json"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

#[test]
fn message_read_with_after_filter() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "after-test");

    post_message(&db_path, &thread_id, "old message");

    // Use a date far in the past - all messages should be included
    let output = cmd()
        .args([
            "message", "read",
            "--thread", &thread_id,
            "--after", "2000-01-01T00:00:00",
            "--format", "json",
        ])
        .env("AIBOARD_DATA_DIR", &db_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1);

    // Use a date far in the future - no messages should match
    let output = cmd()
        .args([
            "message", "read",
            "--thread", &thread_id,
            "--after", "2099-01-01T00:00:00",
            "--format", "json",
        ])
        .env("AIBOARD_DATA_DIR", &db_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 0);
}

#[test]
fn message_read_with_before_filter() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "before-test");

    post_message(&db_path, &thread_id, "recent message");

    // Use a date far in the future - all messages should be included
    let output = cmd()
        .args([
            "message", "read",
            "--thread", &thread_id,
            "--before", "2099-01-01T00:00:00",
            "--format", "json",
        ])
        .env("AIBOARD_DATA_DIR", &db_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1);

    // Use a date in the past - no messages should match
    let output = cmd()
        .args([
            "message", "read",
            "--thread", &thread_id,
            "--before", "2000-01-01T00:00:00",
            "--format", "json",
        ])
        .env("AIBOARD_DATA_DIR", &db_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 0);
}

// --- Cleanup by thread test ---

#[test]
fn cleanup_by_thread() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "cleanup-thread-test");

    post_message(&db_path, &thread_id, "thread message 1");
    post_message(&db_path, &thread_id, "thread message 2");

    // Delete thread via cleanup
    cmd()
        .args(["cleanup", "thread", &thread_id])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    // Thread should be deleted
    cmd()
        .args(["thread", "delete", &thread_id])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .failure();

    // Messages should be gone
    let output = cmd()
        .args(["message", "read", "--thread", &thread_id])
        .env("AIBOARD_DATA_DIR", &db_path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.trim().is_empty());
}

// --- Hook error cases ---

#[test]
fn hook_ingest_invalid_json() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "hook-invalid-json");

    cmd()
        .args(["hook", "ingest", "--thread", &thread_id])
        .write_stdin("not valid json at all{{{")
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .failure();
}

#[test]
fn hook_ingest_unknown_event() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "hook-unknown-event");

    // Valid JSON with unknown hook_event_name - should succeed and store as system message
    let json = serde_json::json!({
        "session_id": "test-session",
        "hook_event_name": "SomeNewEvent"
    });

    cmd()
        .args(["hook", "ingest", "--thread", &thread_id])
        .write_stdin(json.to_string())
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    // Verify the event was stored
    cmd()
        .args(["message", "read", "--thread", &thread_id])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("SomeNewEvent"));
}

#[test]
fn hook_ingest_empty_prompt() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "hook-empty-prompt");

    // UserPromptSubmit with empty prompt - should succeed but ingest 0
    let json = serde_json::json!({
        "session_id": "test-session",
        "hook_event_name": "UserPromptSubmit",
        "prompt": ""
    });

    cmd()
        .args(["hook", "ingest", "--thread", &thread_id])
        .write_stdin(json.to_string())
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();
}

#[test]
fn hook_ingest_user_prompt_submit() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "hook-user-prompt");

    let json = serde_json::json!({
        "session_id": "sess-prompt",
        "hook_event_name": "UserPromptSubmit",
        "transcript_path": "/tmp/test",
        "cwd": "/tmp",
        "prompt": "please fix the bug"
    });

    cmd()
        .args(["hook", "ingest", "--thread", &thread_id])
        .write_stdin(json.to_string())
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    // Verify role=user and content=prompt value
    let output = cmd()
        .args(["message", "read", "--thread", &thread_id, "--format", "json"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["role"], "user");
    assert_eq!(arr[0]["content"], "please fix the bug");
}

#[test]
fn hook_ingest_post_tool_use_skipped() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "hook-post-tool");

    let json = serde_json::json!({
        "session_id": "sess-tool",
        "hook_event_name": "PostToolUse",
        "transcript_path": "/tmp/test",
        "cwd": "/tmp",
        "tool_name": "Bash",
        "tool_input": {"command": "ls -la"},
        "tool_use_id": "tool-123",
        "tool_response": "total 42\ndrwxr-xr-x ..."
    });

    cmd()
        .args(["hook", "ingest", "--thread", &thread_id])
        .write_stdin(json.to_string())
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("0 件"));

    // Verify no messages stored
    let output = cmd()
        .args(["message", "read", "--thread", &thread_id, "--format", "json"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 0);
}

#[test]
fn hook_ingest_stop() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "hook-stop");

    let json = serde_json::json!({
        "session_id": "sess-stop",
        "hook_event_name": "Stop",
        "transcript_path": "/tmp/test",
        "cwd": "/tmp"
    });

    cmd()
        .args(["hook", "ingest", "--thread", &thread_id])
        .write_stdin(json.to_string())
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    // Verify role=assistant, content=[session stop]
    let output = cmd()
        .args(["message", "read", "--thread", &thread_id, "--format", "json"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["role"], "assistant");
    assert_eq!(arr[0]["content"], "[session stop]");
}

#[test]
fn hook_ingest_no_session_no_thread() {
    let (_dir, db_path) = test_db();

    // No --thread and no session_id in JSON -> should fail
    let json = serde_json::json!({
        "hook_event_name": "UserPromptSubmit",
        "prompt": "orphan prompt"
    });

    cmd()
        .args(["hook", "ingest"])
        .write_stdin(json.to_string())
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .failure();
}

// --- Update error cases ---

#[test]
fn update_nonexistent_message() {
    let (_dir, db_path) = test_db();

    cmd()
        .args(["message", "update", "nonexistent-id", "--content", "new content"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .failure();
}

#[test]
fn search_scoped_to_thread() {
    let (_dir, db_path) = test_db();
    let thread_a = create_thread(&db_path, "search-scope-a");
    let thread_b = create_thread(&db_path, "search-scope-b");

    post_message(&db_path, &thread_a, "unique_content_alpha");
    post_message(&db_path, &thread_b, "unique_content_beta");

    // Search scoped to thread A should only find alpha
    cmd()
        .args(["message", "search", "unique_content", "--thread", &thread_a])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta").not());

    // Global search should find both
    cmd()
        .args(["message", "search", "unique_content"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn message_post_all_roles() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "all-roles-test");

    for role in &["user", "assistant", "system", "tool"] {
        cmd()
            .args([
                "message", "post",
                "--thread", &thread_id,
                "--role", role,
                "--content", &format!("{} message", role),
            ])
            .env("AIBOARD_DATA_DIR", &db_path)
            .assert()
            .success();
    }

    let output = cmd()
        .args(["message", "read", "--thread", &thread_id, "--format", "json"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 4);
}

// --- Cleanup backup tests ---

/// Helper: list files matching a glob prefix in a directory.
fn find_backup_files(dir: &str) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("aiboard.db.bak.") {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn cleanup_age_creates_backup_by_default() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "backup-age-test");
    post_message(&db_path, &thread_id, "backup test message");

    // cleanup age without --no-backup should create a backup file
    cmd()
        .args(["cleanup", "age", "0"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("バックアップを作成しました"));

    let backups = find_backup_files(&db_path);
    assert!(!backups.is_empty(), "backup file should be created by default");
}

#[test]
fn cleanup_thread_creates_backup_by_default() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "backup-thread-test");
    post_message(&db_path, &thread_id, "backup thread message");

    cmd()
        .args(["cleanup", "thread", &thread_id])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("バックアップを作成しました"));

    let backups = find_backup_files(&db_path);
    assert!(!backups.is_empty(), "backup file should be created by default");
}

#[test]
fn cleanup_session_creates_backup_by_default() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "backup-session-test");

    cmd()
        .args([
            "message", "post",
            "--thread", &thread_id,
            "--content", "backup session message",
            "--session", "sess-backup",
        ])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    cmd()
        .args(["cleanup", "session", "sess-backup"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("バックアップを作成しました"));

    let backups = find_backup_files(&db_path);
    assert!(!backups.is_empty(), "backup file should be created by default");
}

#[test]
fn cleanup_age_no_backup_skips_backup() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "no-backup-age-test");
    post_message(&db_path, &thread_id, "no backup message");

    cmd()
        .args(["cleanup", "age", "0", "--no-backup"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    let backups = find_backup_files(&db_path);
    assert!(backups.is_empty(), "no backup file should be created with --no-backup");
}

#[test]
fn cleanup_thread_no_backup_skips_backup() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "no-backup-thread-test");
    post_message(&db_path, &thread_id, "no backup thread message");

    cmd()
        .args(["cleanup", "thread", &thread_id, "--no-backup"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    let backups = find_backup_files(&db_path);
    assert!(backups.is_empty(), "no backup file should be created with --no-backup");
}

#[test]
fn cleanup_session_no_backup_skips_backup() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "no-backup-session-test");

    cmd()
        .args([
            "message", "post",
            "--thread", &thread_id,
            "--content", "no backup session message",
            "--session", "sess-no-backup",
        ])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    cmd()
        .args(["cleanup", "session", "sess-no-backup", "--no-backup"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    let backups = find_backup_files(&db_path);
    assert!(backups.is_empty(), "no backup file should be created with --no-backup");
}

#[test]
fn backup_file_naming_format() {
    let (_dir, db_path) = test_db();
    let thread_id = create_thread(&db_path, "naming-format-test");
    post_message(&db_path, &thread_id, "naming format message");

    cmd()
        .args(["cleanup", "age", "0"])
        .env("AIBOARD_DATA_DIR", &db_path)
        .assert()
        .success();

    let backups = find_backup_files(&db_path);
    assert_eq!(backups.len(), 1, "exactly one backup file should be created");

    let name = backups[0].file_name().unwrap().to_str().unwrap();
    // Format: aiboard.db.bak.YYYYMMDDHHmmss (14 digits)
    assert!(name.starts_with("aiboard.db.bak."), "backup name should start with 'aiboard.db.bak.'");
    let timestamp_part = &name["aiboard.db.bak.".len()..];
    assert_eq!(timestamp_part.len(), 14, "timestamp should be 14 digits (YYYYMMDDHHmmss)");
    assert!(timestamp_part.chars().all(|c| c.is_ascii_digit()), "timestamp should be all digits");
}
