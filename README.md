# SortRoot

SortRoot is a desktop drop-zone sorter built with Tauri v2 + React + Tailwind + shadcn-style UI.

It watches a configurable `sortRoot` folder and sorts dropped files/folders into category subfolders:
`Documents`, `Images`, `Video`, `Audio`, `Archives`, `Code`, `Executables`, `Data`, `Misc`.

## Features in this baseline

- Onboarding flow with native folder picker.
- Dashboard with watcher status, run controls, stats, and live activity feed.
- Rules editor with category extension chips and save/revert/export/import.
- Cleanup controls (trash mode, age threshold, protected folders).
- Settings controls for sort root and global sorting toggles.
- Rust backend modules:
  - `rules.rs`
  - `planner.rs`
  - `executor.rs`
  - `cleanup.rs`
  - `watcher.rs`
  - `journal.rs`
  - `errors.rs`
- Tauri command API:
  - `get_rules`, `set_rules`, `validate_rules`, `set_sort_root`
  - `dry_run`, `run_now`, `undo_last_run`
  - `start_watcher`, `stop_watcher`, `watcher_status`
- Event stream:
  - `run_progress`, `run_log`, `watcher_status`

## Development

Prerequisites:
- Node.js 20+
- Rust toolchain
- Tauri desktop prerequisites for your OS

Install dependencies:

```bash
npm install
```

Run frontend build check:

```bash
npm run build
```

Run Rust backend check:

```bash
cd src-tauri
cargo check
```

Run desktop app in dev mode:

```bash
npm run tauri dev
```

## Config persistence

Rules are saved in OS config directory:
- `sort-root/rules.json`

Journal for undo is append-only JSONL:
- `sort-root/journal.jsonl`

## Spec

See `docs/SPEC.md` for behavior contract and acceptance criteria.
