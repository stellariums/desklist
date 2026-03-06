# Desklist

[中文](README.zh-CN.md) | English

A lightweight Windows desktop todo/reminder app. It features a borderless always-on-top window, minimizes to the system tray, and supports scheduled reminders and recurring events.

Desklist is built for people who want a reminder tool that feels fast, visible, and native on Windows. You can open it in seconds, capture tasks quickly, keep it quietly in the tray, and rely on timed or recurring reminders without switching to a heavy project-management app.

Compared with bloated todo apps, Desklist focuses on the desktop experience: a clean frosted-glass UI, quick list and calendar views, local SQLite storage, and lightweight daily planning that stays close to your workflow.

## Features

- Create, edit, and delete events with title, description, and time
- Four filter views: Today / Incomplete / Completed / All
- **Calendar view**: switch between list and calendar views via the title bar button; dates with pending events show a dot; click a date to see that day's events
- Scheduled reminders: due-time reminders + optional advance reminders (5/15/30/60 minutes or 1 day before)
- Recurring events: daily, weekly, monthly, with optional end time
- Automatically generates the next occurrence after completing a recurring event
- Runs in the system tray and can hide to tray when the window is closed
- Single-instance application
- Dark frosted-glass UI (Windows Acrylic)

## Tech Stack

- Frontend: Vue 3 + TypeScript + Vite
- Desktop framework: Tauri 2
- Data storage: SQLite (`tauri-plugin-sql`)
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

## Changelog

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

