-- Schema v2: Add source column to messages

ALTER TABLE messages ADD COLUMN source TEXT;

CREATE INDEX IF NOT EXISTS idx_messages_source ON messages(source);

INSERT INTO schema_version (version) VALUES (2);
