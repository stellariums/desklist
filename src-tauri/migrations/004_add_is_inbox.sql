ALTER TABLE events ADD COLUMN is_inbox INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_events_is_inbox ON events(is_inbox, deleted_at);
