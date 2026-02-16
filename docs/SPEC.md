# SecondBreakfast Specification

## Product Summary
SecondBreakfast is a drop-zone sorter desktop app built with Tauri v2 (Rust backend) and React frontend. Users choose a configurable `SecondBreakfast` folder. The app watches that folder and automatically sorts dropped files and folder trees into category subfolders.

## Category Subfolders
The app ensures these protected subfolders exist under `SecondBreakfast`:
- `Documents`
- `Images`
- `Video`
- `Audio`
- `Archives`
- `Code`
- `Executables`
- `Data`
- `Misc`

These folders are protected and must never be deleted by cleanup.

## Sorting Rules
- Sort mode: flatten (no source directory preservation).
- Classification is extension-based.
- Unknown extensions route to `Misc` when `unknownGoesToMisc = true`.
- Files with no extension route to `Misc` when `noExtensionGoesToMisc = true`.
- Collision policy for MVP: rename (`name.ext`, `name (1).ext`, `name (2).ext`).
- Safety gate: only move files older than `minFileAgeSeconds`.

## Cleanup Rules
- Optional empty-folder cleanup runs after sorting.
- Traversal is bottom-up (post-order).
- Deletion mode is Trash/Recycle only (`mode = "trash"`).
- Respect directory age threshold (`minDirAgeSeconds`).
- Never delete:
  - `SecondBreakfast`
  - protected category folders
  - any descendants of protected category folders

## Config Persistence
Config lives in the OS app config directory as `rules.json` and includes:
- `global`:
  - `SecondBreakfast`
  - `caseInsensitiveExt`
  - `collisionPolicy`
  - `unknownGoesToMisc`
  - `noExtensionGoesToMisc`
  - `minFileAgeSeconds`
  - `cleanupEmptyFolders`:
    - `enabled`
    - `minAgeSeconds`
    - `mode` (`trash`)
- `categories[]`:
  - `id`
  - `name`
  - `targetSubfolder`
  - `extensions[]`
- `misc`:
  - `name`
  - `targetSubfolder`

## Backend Modules
- `errors.rs`: centralized app error types.
- `rules.rs`: defaults, load/save, validate, extension lookup.
- `planner.rs`: scan and build executable plan + dry-run preview.
- `executor.rs`: execute moves, collision renames, progress emission.
- `cleanup.rs`: safe empty-folder trash cleanup.
- `watcher.rs`: notify watcher with debounce and start/stop status.
- `journal.rs`: JSONL journal and undo-last-run.

## Command Contract
Commands:
- `get_rules() -> Rules`
- `set_rules(rules) -> ()`
- `validate_rules(rules) -> ValidationResult`
- `set_sort_root(path) -> ()`
- `dry_run() -> PlanPreview`
- `run_now() -> RunResult`
- `undo_last_run() -> UndoResult`
- `start_watcher() -> ()`
- `stop_watcher() -> ()`
- `watcher_status() -> WatcherStatus`

Events:
- `run_progress { moved, skipped, errors, currentPath, destPath }`
- `run_log { level, message }`
- `watcher_status { running, SecondBreakfast }`

## UI Screens
- Onboarding: pick sort folder and start watcher.
- Dashboard: watcher status, run actions, activity feed, summary.
- Rules: editable category extension chips and global toggles.
- Cleanup: cleanup toggle and age settings.
- Settings: sort root change, protected folders, rules import/export.

## Acceptance Tests
1. Sort folder can be selected, persisted, and changed later.
2. Mixed dropped file trees are flattened into correct categories.
3. Unknown and no-extension files route to `Misc` by default.
4. `SecondBreakfast` and category folders are never deleted.
5. Empty remnants are trashed after a run.
6. Dry run destination/counts match real run behavior.
7. Undo-last-run restores moved files best-effort and reports skips.
