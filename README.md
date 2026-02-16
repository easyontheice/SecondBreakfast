# 🥖 SecondBreakfast

**v1.0.0 — Officially out of the Shire.**

SecondBreakfast is a real-time drop-folder organizer inspired by the comforts of a well-ordered Hobbit hole.

Choose a **Sort Folder**, and the app keeps watch like a dutiful Hobbit, automatically sorting any files or folders dropped inside into neat category subfolders — all within the same directory. Reliable, unobtrusive, and perfectly content to work quietly while you enjoy second breakfast.

Built with **Tauri v2 + React + Tailwind + shadcn-style UI**, powered by a Rust backend that takes order very seriously.

---

## 🍃 What It Does

SecondBreakfast watches a configurable `sortRoot` and automatically routes incoming files into:

* `Documents`
* `Images`
* `Video`
* `Audio`
* `Archives`
* `Code`
* `Executables`
* `Data`
* `Misc`

Drop a single file.
Drop an entire folder tree.
Walk away.

Everything finds its proper place.

---

## 🌿 Features (v1.0.0)

### 🧭 Onboarding

* Native folder picker
* Clean first-run experience
* Watcher auto-start setup

### 📊 Dashboard

* Watcher status indicator
* Run controls
* Live progress + activity feed
* Undo last run

### 🏷 Rules Editor

* Category extension chips
* Add/remove extensions
* Save / revert
* Export / import `rules.json`

### 🧹 Cleanup Controls

* Optional empty-folder cleanup
* Protected category folders
* Trash mode support

### ⚙ Settings

* Change sort root anytime
* Global sorting toggles

### 🦀 Rust Backend Modules

* `rules.rs`
* `planner.rs`
* `executor.rs`
* `cleanup.rs`
* `watcher.rs`
* `journal.rs`
* `errors.rs`

### 🔌 Tauri Command API

* `get_rules`, `set_rules`, `validate_rules`, `set_sort_root`
* `dry_run`, `run_now`, `undo_last_run`
* `start_watcher`, `stop_watcher`, `watcher_status`

### 📡 Event Stream

* `run_progress`
* `run_log`
* `watcher_status`
* `run_complete`

---

## 🔁 Undo System (Now with Restored Folder)

Undo no longer drops files back into unpredictable locations.

Instead, restored files are placed under:

```
<sortRoot>/Restored/<session_id>/...
```

Preserving their original directory structure in a safe, isolated location.

Because even Hobbits believe in second chances.

---

## 🛠 Development

### Prerequisites

* Node.js 20+
* Rust toolchain
* Tauri desktop prerequisites for your OS

Install dependencies:

```bash
npm install
```

Frontend build check:

```bash
npm run build
```

Rust backend check:

```bash
cd src-tauri
cargo check
```

Run acceptance tests:

```bash
cd src-tauri
cargo test acceptance_ -- --nocapture
```

Run desktop app in dev mode:

```bash
npm run tauri dev
```

---

## 🚀 Releases

### v1.0.0

* Stable real-time watcher
* Deterministic undo system with `Restored/<session_id>` isolation
* Acceptance-tested planner + executor
* Config persistence via OS config directory
* Windows installer auto-build via GitHub Actions

---

## 🏷 Version Bump Workflow

Local:

```bash
npm run bump:version -- 1.0.1
```

Updates:

* `package.json`
* `src-tauri/Cargo.toml`
* `src-tauri/tauri.conf.json`

---

## 📦 Auto-Release Workflow

Push a version tag (example `v1.0.0`) and GitHub Actions will build and publish Windows installers automatically.

Workflow:

```
.github/workflows/release-windows.yml
```

Published assets:

* `.msi` installer
* `-setup.exe` installer

---

## 💾 Config Persistence

Rules are stored in your OS config directory:

```
sort-root/rules.json
```

Undo journal is append-only JSONL:

```
sort-root/journal.jsonl
```

---

## 📜 Documentation

See:

* `docs/BUILD.md`
* `docs/ACCEPTANCE.md`
* `docs/RELEASE_CHECKLIST.md`
* `docs/THEME_TOKENS.md`
* `docs/SPEC.md`

