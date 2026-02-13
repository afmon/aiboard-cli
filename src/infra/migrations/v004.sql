-- Schema v4: Add phase column to threads (workflow phase tracking)

ALTER TABLE threads ADD COLUMN phase TEXT DEFAULT NULL;

INSERT INTO schema_version (version) VALUES (4);
