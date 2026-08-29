# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Vue 3 + TypeScript frontend.
- `src/components/`: UI components (for example `EventList.vue`, `CalendarView.vue`).
- `src/composables/`: state and data logic (`useEvents.ts`, theme/locale/settings hooks).
- `src/i18n/`: locale dictionaries (`en.ts`, `zh-CN.ts`).
- `src/styles/`: global styles.
- `src-tauri/`: Rust desktop host for Tauri.
- `src-tauri/src/`: backend modules (`lib.rs`, `db.rs`, `events.rs`, `scheduler.rs`, `tray.rs`).
- `src-tauri/migrations/`: SQLite schema migrations.
- `public/`: static assets; `dist/` is build output (do not edit manually).

## Build, Test, and Development Commands
- `npm install`: install frontend/tooling dependencies.
- `npm run dev`: start Vite frontend only.
- `npm run build`: run `vue-tsc` and production frontend build.
- `npm run tauri dev`: run full desktop app in development.
- `npm run tauri build`: build distributable desktop app.
- `cd src-tauri && cargo check`: validate Rust code compiles.
- `cd src-tauri && cargo test`: run Rust business-logic tests against temporary data.

## Coding Style & Naming Conventions
- Use 2-space indentation in TypeScript/Vue and keep `<script setup lang="ts">` style.
- Use `PascalCase` for component files, `camelCase` for functions/composables, and `snake_case` for DB fields.
- In Rust, keep module/function names idiomatic (`snake_case`) and prefer explicit error propagation over panics.
- No strict formatter config is enforced; follow existing file style and keep diffs minimal.

## Testing Guidelines
- Rust tests cover recurrence and the event lifecycle using an in-memory database.
- Minimum validation for code changes:
  - `npm run build`
  - `cd src-tauri && cargo test`
  - `cd src-tauri && cargo check`
- For behavior changes, run manual smoke checks in `npm run tauri dev` (data-folder selection, event create/edit, recurring completion, reminder behavior, calendar rendering).
- Never run destructive tests against the user's selected `desklist.db`.

## Commit & Pull Request Guidelines
- Follow commit style seen in history: `fix: ...`, `docs: ...`, `chore(release): ...`.
- Use concise, imperative commit messages and scope when helpful.
- PRs should include:
  - change summary and motivation,
  - verification steps run locally,
  - screenshots/GIFs for UI changes,
  - migration notes when `src-tauri/migrations/` is touched.

## Security & Configuration Notes
- Keep Tauri capability and CSP changes minimal and justified.
- Never commit secrets, tokens, or local environment-specific files.
- Add new numbered migration files; do not rewrite already released migrations.
- Never commit SQLite databases, `data-location.json`, or machine-specific data paths.
