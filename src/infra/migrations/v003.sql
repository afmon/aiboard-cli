-- Schema v3: Add status column to threads (open/closed)

ALTER TABLE threads ADD COLUMN status TEXT NOT NULL DEFAULT 'open' CHECK(status IN ('open', 'closed'));

CREATE INDEX IF NOT EXISTS idx_threads_status ON threads(status);

INSERT INTO schema_version (version) VALUES (3);
