# Desklist

[中文](README.zh-CN.md) | English

A lightweight Windows desktop todo/reminder app. It features a borderless desktop-bottom window, minimizes to the system tray, and supports scheduled reminders and recurring events.

Desklist is built for people who want a reminder tool that feels fast, visible, and native on Windows. You can open it in seconds, capture tasks quickly, keep it quietly in the tray, and rely on timed or recurring reminders without switching to a heavy project-management app.

Compared with bloated todo apps, Desklist focuses on the desktop experience: a clean frosted-glass UI, quick list and calendar views, local SQLite storage, and lightweight daily planning that stays close to your workflow.

## Features

- Create, edit, and safely delete events with title, description, a scheduled time block, and an optional deadline
- Four filter views: Today / Incomplete / Completed / All
- **Calendar view**: switch between list and calendar views via the title bar button; dates with pending events show a dot; click a date to see that day's events
- Scheduled reminders: due-time reminders + optional advance reminders (5/15/30/60 minutes or 1 day before)
- Recurring events: daily, weekly, monthly, with optional end time
- Automatically generates the next occurrence after completing a recurring event
- Runs in the system tray and can hide to tray when the window is closed
- Single-instance application
- Dark frosted-glass UI (Windows Acrylic)
- Choose the task data folder on first launch instead of being forced to use the system drive
- View and open the active data folder from Settings
- Local browser workbench for viewing, creating, editing, completing, and reopening tasks
- Inbox for capturing tasks before assigning a date and time
- Today home page combining today's agenda, historical overdue tasks, and inbox status
- Weekly and monthly review calendars with period navigation, a Today shortcut, and synchronized daily details

## Data Storage

On first launch, Desklist asks where to store task data. If an older database is found, the app copies it to the selected folder, verifies the copy, and only then enables the new location. The original database is not deleted automatically.

App upgrades or reinstalls do not overwrite the selected data folder. Do not manually move or edit `desklist.db`, `desklist.db-wal`, or `desklist.db-shm` while Desklist is running.

## Tech Stack

- Frontend: Vue 3 + TypeScript + Vite
- Desktop framework: Tauri 2
- Data storage: SQLite (managed centrally in Rust with `sqlx`)
- Reminder scheduling: Rust + Tokio
- Notifications: `tauri-plugin-notification`

## Development

```bash
# Install dependencies
npm install

# Development mode
npm run tauri dev

# Production build
npm run tauri build
```

## Roadmap

The browser-based personal workbench, protected local Agent REST API, and Streamable HTTP MCP are complete. The next phase is real Agent integration testing. See the [personal workbench roadmap](docs/ROADMAP.zh-CN.md) for completed work, implementation order, and deferred scope.

## Browser workbench

While Desklist is running, choose `Open Browser Workbench` from the tray menu or visit `http://127.0.0.1:47831`. The workbench can capture unscheduled tasks in the inbox, schedule them into the calendar, set a separate optional deadline, manage task completion, and safely delete or restore tasks through the recycle bin.

The web service listens on the local computer only and is not available to other devices on the network.

## Changelog

### v1.6.0

- Added a token-protected local Agent REST API and OpenAPI contract
- Added a Streamable HTTP MCP server with list and single-task queries, capture, create, update, completion, reopening, and recycle-bin tools
- Added Agent endpoint and token management to desktop settings
- Made Agent complete/reopen operations idempotent and limited deletion to the recoverable recycle bin
- Added an audit log for Agent writes, including task, action, source, and time
- Added local-only browser task endpoints for viewing, creating, editing, completing, and reopening tasks
- Added a recycle bin with soft deletion, restore, and confirmed permanent deletion
- Added an inbox for quick capture and later scheduling into the calendar
- Added a Today home page with agenda, overdue-task, and inbox summaries plus quick actions
- Separated scheduled start/end times from optional task deadlines; reminders target the deadline when one is set
- Added weekly/monthly calendar switching with cross-month week navigation and synchronized daily review details
- Refined the browser workbench's editorial visual system, responsive spacing, focus states, empty states, and task modal behavior
- Added a tray shortcut for opening the browser workbench
- Added first-launch data-folder selection and safe migration of an existing database
- Added the active data location to Settings with a shortcut to open the folder
- Centralized task CRUD, recurrence, and reminder behavior in the Rust backend
- Added temporary-database tests so verification never changes real user tasks

### v1.5.1

- Fixed window z-order behavior: Desklist now stays at the desktop bottom layer and can be covered by normal app windows (browsers, editors, etc.)

### v1.5.0

- Fixed duplicate next-occurrence generation when repeatedly toggling recurring tasks between complete/incomplete
- Fixed monthly recurrence rollover for end-of-month dates (for example, Jan 31 now rolls to Feb 28 instead of skipping to March)
- Fixed reminder queue behavior when toggling completion: unfired reminders are cleaned up on complete, and only future reminders are regenerated when reopening
- Fixed window layering to match product behavior (`alwaysOnTop`)
- Fixed create-from-calendar default time: after selecting a date in calendar view, new events now default to that selected date instead of today
- Fixed Chinese month/date rendering in calendar view to prevent malformed template text from appearing in the UI

### v1.4.0

- Added calendar view: switch between list and calendar via the title bar button
- Dates with pending events display a blue dot; today's date is highlighted
- Click a date to show that day's pending events below the grid, with complete/delete/edit support
- Navigate months with `<` / `>` buttons; dots refresh automatically
- Month labels adapt to language (e.g. "March 2026" in English, "2026年3月" in Chinese)

### v1.3.0

- Added language switching (Chinese / English) in the settings panel
- All UI text now responds to language changes instantly, with preference persisted across restarts
- **Security:** Added Content Security Policy (CSP) to the Tauri WebView
- **Security:** Fixed high-severity CVEs in `minimatch` (ReDoS x3) and `rollup` (path traversal) via dependency upgrades
- **Security:** Accent color value is now validated before being applied as a CSS property
- **Security:** Removed unused `opener` capability from the permission manifest
- **Security:** Added `maxlength` constraints (200 / 1000 chars) on title and description inputs
- **Security:** `recurrence` field is now validated against an allowlist before being written to the database
- **Stability:** Replaced `unwrap()` with proper error propagation in the Tauri setup path

### v1.2.1

- Changed list filter from "Upcoming" to "Incomplete" to show all unfinished tasks
- Added a setting to configure whether due-time reminders are enabled by default for new events
- Fixed garbled Chinese text in multiple UI areas (new event, reminder configuration, settings panel, README)

### v1.2.0

- Added an appearance settings panel (accessible from the gear button in the title bar)
- Added adjustable window opacity (10%~100%)
- Added customizable theme color
- Persisted settings with `tauri-plugin-store`

### v1.1.0

- Refactored dark frosted-glass style and enabled Windows Acrylic window effects
- Unified dark-theme support across major components

### v1.0.2

- Fixed notification time display issues
- Improved UI visuals and interaction details

### v1.0.1

- Fixed reminder notifications playing no sound; the system default alert sound now plays when a reminder fires (#1)

### v1.0.0

- Initial release

## License

[MIT](LICENSE)

