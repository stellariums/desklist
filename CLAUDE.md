# CLAUDE.md

This file provides guidance to coding agents working in this repository.

## Build and Test Commands

```bash
npm run tauri dev      # Full app development mode
npm run tauri build    # Production frontend, Rust binary, and NSIS installer
npm run build          # Frontend type-check and Vite build
cargo check            # Rust compile check, run from src-tauri/
cargo test             # Rust behavior tests, run from src-tauri/
```

## Current Architecture

Desklist is a Windows todo/reminder app built with Tauri 2, Vue 3, TypeScript, Rust, and SQLite. It has a frameless 360×520 Acrylic desktop shell plus a local-only browser workbench. The browser workbench foundation is complete; the next product phase, documented in `docs/ROADMAP.zh-CN.md`, is protected local Agent interfaces.

### Data Flow

- Vue does not access SQLite directly. `src/composables/useEvents.ts` calls Tauri commands.
- `src-tauri/src/events.rs` owns event CRUD, validation, recurrence generation, and reminder-queue updates.
- `src-tauri/src/db.rs` owns the selected data directory, SQLite pool, migrations, first-run migration, and database checks.
- `src-tauri/src/scheduler.rs` uses the same shared SQLite pool and polls reminders every 30 seconds.
- `src-tauri/src/web_server.rs` serves embedded Vue assets and local task query/create/inbox/update/toggle/trash/restore/delete endpoints on `127.0.0.1:47831`.
- HTTP handlers stay thin and reuse the Rust event business-logic layer; future MCP handlers must follow the same rule.

### Data Location and First-Run Migration

- The user chooses a data directory on first launch through `DataLocationSetup.vue`.
- A small locator file remains at the Tauri app config directory as `data-location.json`; it stores only the selected directory path.
- The active database is `<selected directory>/desklist.db`. SQLite may also create `desklist.db-wal` and `desklist.db-shm` beside it.
- If the legacy AppData database exists, `configure_data_directory` copies it with SQLite `VACUUM INTO`, validates integrity and core row counts, then atomically enables the new database.
- The legacy database is deliberately retained until the user explicitly approves deletion. Never add an automatic fallback that silently creates another database in AppData.
- If the selected drive or database is unavailable, surface an error instead of switching data locations.

### Database and Migrations

- SQLite is managed by a shared `sqlx::SqlitePool` in `DatabaseState`.
- Migration SQL lives in `src-tauri/migrations/` and runs through `sqlx::migrate!`.
- Existing tables are `events` and `reminder_queue`; `events.deleted_at` implements the recycle bin and `events.is_inbox` keeps unscheduled captures out of calendars and reminders. `event_time` is the scheduled start, while `scheduled_end` and `due_time` are optional; existing tasks keep their start time and receive no invented end or deadline. Released migrations must never be rewritten.
- Rust SQL uses `?` placeholders.
- Event timestamps are UTC RFC 3339 strings; list and calendar boundaries are calculated in local time by the frontend and passed as UTC strings.
- On-time and advance reminders are anchored to `due_time` when present, otherwise to `event_time`. Recurring occurrences shift optional schedule-end, deadline, and advance-reminder timestamps by the same offset as the scheduled start.

### Tauri Commands

Data setup:

- `get_data_status`
- `configure_data_directory`
- `open_data_directory`

Event operations:

- `fetch_events`
- `fetch_month_events`
- `create_event`
- `create_inbox_event`
- `update_event`
- `delete_event`
- `restore_event`
- `permanently_delete_event`
- `toggle_complete`

### Frontend Structure

- `App.vue` — switches between the Tauri desktop shell and browser workbench
- `components/BrowserWorkbench.vue` — responsive browser workbench with Today, weekly/monthly review calendars, inbox, task operations, and recycle-bin flows
- `components/DataLocationSetup.vue` — first-run folder selection and migration UI
- `components/EventList.vue` / `CalendarView.vue` — list and calendar views
- `components/EventForm.vue` — create/edit panel
- `components/SettingsPanel.vue` — appearance, language, reminder defaults, and active data-folder shortcut
- `composables/useEvents.ts` — typed Tauri-command adapter; contains no SQL or recurrence rules
- `types/index.ts` — shared frontend types

### Rust Modules

- `lib.rs` — Tauri builder, command registration, close-to-tray setup
- `db.rs` — data-location state, database connection, migration, and validation
- `events.rs` — event business logic and temporary-database tests
- `scheduler.rs` — reminder polling and notifications
- `tray.rs` — system tray menu and handlers
- `web_server.rs` — local HTTP listener, task query/write endpoints, and embedded frontend assets

### Close-to-Tray

`TitleBar.vue` hides the window for its minimize/close buttons. Rust also intercepts OS-level `CloseRequested`, prevents exit, and hides the window. The tray Quit command is the explicit exit path.

## Verification Expectations

For every code change, run:

```bash
npm run build
cd src-tauri
cargo test
cargo check
```

For packaging, data-location, or browser-service changes, also run `npm run tauri build` and verify that the app restarts without Vite, reads the selected database, serves `http://127.0.0.1:47831`, and leaves the legacy database untouched.

Tests that create, edit, complete, or delete events must use a temporary/in-memory database. Never use the user's real `desklist.db` as test data.

## Important Constraints

- Preserve current recurring-event idempotency and month-end behavior.
- Keep frontend title/description limits aligned with Rust validation: 200/1000 characters.
- Keep capabilities and CSP minimal.
- Do not commit databases, locator files, local data paths, tokens, build output, or local environment files.
- `tauri-plugin-store` remains responsible for lightweight theme, locale, and app settings only; task data belongs in SQLite.
