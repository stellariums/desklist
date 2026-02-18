CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT DEFAULT '',
    event_time TEXT NOT NULL,
    completed INTEGER DEFAULT 0,
    remind_at TEXT,
    remind_on_time INTEGER DEFAULT 1,
    recurrence TEXT DEFAULT 'none',
    recurrence_end TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS reminder_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL,
    fire_at TEXT NOT NULL,
    fired INTEGER DEFAULT 0,
    type TEXT NOT NULL DEFAULT 'on_time',
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_reminder_queue_fire_at ON reminder_queue(fire_at);
CREATE INDEX IF NOT EXISTS idx_reminder_queue_unfired ON reminder_queue(fired, fire_at);
CREATE INDEX IF NOT EXISTS idx_events_event_time ON events(event_time);
CREATE INDEX IF NOT EXISTS idx_events_completed ON events(completed);
