# Repository Guidelines

## Project Structure & Module Organization
Desklist is a Tauri desktop app with a Vue frontend.

- `src/`: Vue 3 + TypeScript UI (`components/`, `composables/`, `types/`, `styles/`).
- `src-tauri/src/`: Rust backend modules (`lib.rs`, `tray.rs`, `scheduler.rs`, `db.rs`).
- `src-tauri/migrations/`: SQLite schema migration files (for example `001_init.sql`).
- `public/`: static frontend assets.
- `.github/workflows/`: CI automation and review workflows.

Keep frontend data logic in `src/composables/useEvents.ts`; keep OS/system integration in Rust.

## Build, Test, and Development Commands
- `npm install`: install Node dependencies.
- `npm run tauri dev`: run full desktop app in development (Vite + Tauri).
- `npm run build`: frontend type-check and production build (`vue-tsc --noEmit && vite build`).
- `npm run tauri build`: build installable desktop bundle (NSIS target).
- `cargo check` (run in `src-tauri/`): compile-check Rust backend quickly.
- `npx vue-tsc --noEmit`: standalone frontend type check.

## Coding Style & Naming Conventions
- TypeScript uses strict mode (`tsconfig.json`); fix all type errors before PR.
- Use 2-space indentation in Vue/TS/CSS; keep existing formatting style.
- Vue SFCs and components: PascalCase file names (for example `EventList.vue`).
- Composables: `useXxx.ts` (for example `useTheme.ts`).
- Rust modules/functions follow idiomatic `snake_case`; types use `CamelCase`.

No dedicated lint config is committed yet; rely on TypeScript checks and consistent existing style.

## Testing Guidelines
There is currently no formal unit/integration test suite in this repository.

Minimum validation for contributions:
- Run `npm run build`.
- Run `cargo check` in `src-tauri/`.
- Manually verify key flows in `npm run tauri dev` (event CRUD, reminder behavior, tray interactions).

If adding tests, place frontend tests under `src/**/__tests__/` and Rust tests near modules (`mod tests`).

## Commit & Pull Request Guidelines
Follow the existing commit pattern: Conventional Commits with concise scopes, e.g.:
- `feat: add reminder snooze option`
- `fix: resolve tray menu visibility issue`
- `docs: update README changelog`

PRs should include:
- Clear summary of user-visible and technical changes.
- Linked issue (if applicable).
- Verification notes with commands run.
- UI screenshots/GIFs for visual changes (Vue components, theme, window behavior).

Keep PRs focused; avoid unrelated refactors.
