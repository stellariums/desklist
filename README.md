# Desklist

[中文](README.zh-CN.md) | English

A lightweight Windows desktop todo/reminder app. It features a borderless always-on-top window, minimizes to the system tray, and supports scheduled reminders and recurring events.

## Features

- Create, edit, and delete events with title, description, and time
- Four filter views: Today / Incomplete / Completed / All
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
