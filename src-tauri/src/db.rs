use tauri_plugin_sql::{Migration, MigrationKind};

pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create events and reminder_queue tables",
            sql: include_str!("../migrations/001_init.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add generated_next_id for recurring event idempotency",
            sql: include_str!("../migrations/002_add_generated_next_id.sql"),
            kind: MigrationKind::Up,
        },
    ]
}
