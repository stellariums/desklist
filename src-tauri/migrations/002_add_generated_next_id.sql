ALTER TABLE events ADD COLUMN generated_next_id TEXT;

CREATE INDEX IF NOT EXISTS idx_events_generated_next_id ON events(generated_next_id);
