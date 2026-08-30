ALTER TABLE events ADD COLUMN scheduled_end TEXT;
ALTER TABLE events ADD COLUMN due_time TEXT;

CREATE INDEX IF NOT EXISTS idx_events_scheduled_end ON events(scheduled_end);
CREATE INDEX IF NOT EXISTS idx_events_due_time ON events(due_time);
