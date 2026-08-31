CREATE TABLE IF NOT EXISTS agent_audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT,
    action TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agent_audit_log_created_at
ON agent_audit_log(created_at);

CREATE INDEX IF NOT EXISTS idx_agent_audit_log_event_id
ON agent_audit_log(event_id);
