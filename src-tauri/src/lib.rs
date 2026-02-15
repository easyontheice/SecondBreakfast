mod cleanup;
mod errors;
mod executor;
mod journal;
mod planner;
mod rules;
mod watcher;

use crate::errors::{AppError, AppResult};
use cleanup::CleanupResult;
use executor::RunResult;
use planner::PlanPreview;
use rules::{Rules, ValidationResult};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use watcher::{DebouncedAction, WatcherController, WatcherStatus};

#[derive(Clone)]
struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    rules: Mutex<Rules>,
    rules_path: PathBuf,
    journal_path: PathBuf,
    watcher: Arc<Mutex<WatcherController>>,
    pipeline_running: AtomicBool,
}

impl AppState {
    fn new(rules: Rules, rules_path: PathBuf, journal_path: PathBuf) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                rules: Mutex::new(rules),
                rules_path,
                journal_path,
                watcher: Arc::new(Mutex::new(WatcherController::default())),
                pipeline_running: AtomicBool::new(false),
            }),
        }
    }

    fn current_rules(&self) -> AppResult<Rules> {
        Ok(self.inner.rules.lock()?.clone())
    }

    fn replace_rules(&self, next: Rules) -> AppResult<()> {
        *self.inner.rules.lock()? = next;
        Ok(())
    }

    fn watcher_running(&self) -> AppResult<bool> {
        Ok(self.inner.watcher.lock()?.running)
    }
}

struct RunGuard<'a> {
    flag: &'a AtomicBool,
}

impl<'a> RunGuard<'a> {
    fn acquire(flag: &'a AtomicBool) -> AppResult<Self> {
        if flag.swap(true, Ordering::SeqCst) {
            return Err(AppError::State("a run is already in progress".to_string()));
        }
        Ok(Self { flag })
    }
}

impl Drop for RunGuard<'_> {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

#[tauri::command]
fn get_rules(state: State<AppState>) -> Result<Rules, String> {
    map_err(state.current_rules())
}

#[tauri::command]
fn set_rules(state: State<AppState>, rules: Rules) -> Result<(), String> {
    map_err(set_rules_internal(state.inner(), rules))
}

#[tauri::command]
fn validate_rules(rules: Rules) -> ValidationResult {
    rules::validate_rules(&rules)
}

#[tauri::command]
fn set_sort_root(app: AppHandle, state: State<AppState>, path: String) -> Result<(), String> {
    map_err(set_sort_root_internal(&app, state.inner(), path))
}

#[tauri::command]
fn dry_run(state: State<AppState>) -> Result<PlanPreview, String> {
    map_err(dry_run_internal(state.inner()))
}

#[tauri::command]
fn run_now(app: AppHandle, state: State<AppState>) -> Result<RunResult, String> {
    map_err(run_now_internal(&app, state.inner()))
}

#[tauri::command]
fn undo_last_run(app: AppHandle, state: State<AppState>) -> Result<journal::UndoResult, String> {
    map_err(undo_last_run_internal(&app, state.inner()))
}

#[tauri::command]
fn start_watcher(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    map_err(start_watcher_internal(&app, state.inner()))
}

#[tauri::command]
fn stop_watcher(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    map_err(stop_watcher_internal(&app, state.inner()))
}

#[tauri::command]
fn watcher_status(state: State<AppState>) -> Result<WatcherStatus, String> {
    map_err(watcher_status_internal(state.inner()))
}

fn set_rules_internal(state: &AppState, rules: Rules) -> AppResult<()> {
    let validation = rules::validate_rules(&rules);
    if !validation.valid {
        return Err(AppError::Validation(validation.errors.join("; ")));
    }

    rules::ensure_sort_root_dirs(&rules)?;
    rules::save_rules(&state.inner.rules_path, &rules)?;
    state.replace_rules(rules)?;
    Ok(())
}

fn set_sort_root_internal(app: &AppHandle, state: &AppState, path: String) -> AppResult<()> {
    let mut rules = state.current_rules()?;
    rules.global.sort_root = path;

    set_rules_internal(state, rules)?;

    if state.watcher_running()? {
        stop_watcher_internal(app, state)?;
        start_watcher_internal(app, state)?;
    }
    Ok(())
}

fn dry_run_internal(state: &AppState) -> AppResult<PlanPreview> {
    let rules = state.current_rules()?;
    rules::ensure_sort_root_dirs(&rules)?;
    planner::build_plan(&rules)
}

fn run_now_internal(app: &AppHandle, state: &AppState) -> AppResult<RunResult> {
    let _guard = RunGuard::acquire(&state.inner.pipeline_running)?;
    let rules = state.current_rules()?;

    rules::ensure_sort_root_dirs(&rules)?;

    let plan = planner::build_plan(&rules)?;
    let mut result = executor::execute_plan(app, &plan)?;

    if rules.global.cleanup_empty_folders.enabled {
        let cleanup_result = cleanup::cleanup_empty_folders(&rules)?;
        apply_cleanup(&mut result, cleanup_result);
    }

    journal::append_run(&state.inner.journal_path, &result.session_id, &result.moved_files)?;
    Ok(result)
}

fn undo_last_run_internal(app: &AppHandle, state: &AppState) -> AppResult<journal::UndoResult> {
    let result = journal::undo_last_run(&state.inner.journal_path)?;
    executor::emit_log(
        app,
        "info",
        format!(
            "undo complete: restored={}, skipped={}, errors={}",
            result.restored, result.skipped, result.errors
        ),
    );
    Ok(result)
}

fn start_watcher_internal(app: &AppHandle, state: &AppState) -> AppResult<()> {
    let rules = state.current_rules()?;
    rules::ensure_sort_root_dirs(&rules)?;

    let sort_root = PathBuf::from(&rules.global.sort_root);
    let app_handle = app.clone();
    let state_clone = state.clone();
    let action: DebouncedAction = Arc::new(move || {
        if let Err(err) = run_now_internal(&app_handle, &state_clone) {
            executor::emit_log(&app_handle, "error", format!("watcher-triggered run failed: {}", err));
        }
    });

    watcher::start_watcher(
        &state.inner.watcher,
        sort_root,
        Duration::from_secs(2),
        action,
    )?;

    emit_watcher_status(app, state)
}

fn stop_watcher_internal(app: &AppHandle, state: &AppState) -> AppResult<()> {
    watcher::stop_watcher(&state.inner.watcher)?;
    emit_watcher_status(app, state)
}

fn watcher_status_internal(state: &AppState) -> AppResult<WatcherStatus> {
    let rules = state.current_rules()?;
    let running = state.watcher_running()?;
    Ok(WatcherStatus {
        running,
        sort_root: rules.global.sort_root,
    })
}

fn emit_watcher_status(app: &AppHandle, state: &AppState) -> AppResult<()> {
    let status = watcher_status_internal(state)?;
    let _ = app.emit("watcher_status", status);
    Ok(())
}

fn apply_cleanup(result: &mut RunResult, cleanup: CleanupResult) {
    result.cleanup_trashed = cleanup.trashed;
    result.cleanup_errors = cleanup.errors;
}

fn map_err<T>(result: AppResult<T>) -> Result<T, String> {
    result.map_err(|err| err.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let rules_path = rules::rules_path()?;
            let journal_path = rules::journal_path()?;
            let rules = rules::load_or_create_rules(&rules_path)?;
            rules::ensure_sort_root_dirs(&rules)?;

            app.manage(AppState::new(rules, rules_path, journal_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_rules,
            set_rules,
            validate_rules,
            set_sort_root,
            dry_run,
            run_now,
            undo_last_run,
            start_watcher,
            stop_watcher,
            watcher_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod acceptance_tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};
    use uuid::Uuid;

    fn temp_sort_root() -> PathBuf {
        std::env::temp_dir().join(format!("sortroot-acceptance-{}", Uuid::new_v4()))
    }

    fn make_rules(root: &Path) -> rules::Rules {
        let mut rules = rules::default_rules();
        rules.global.sort_root = root.to_string_lossy().to_string();
        rules.global.min_file_age_seconds = 0;
        rules.global.cleanup_empty_folders.enabled = true;
        rules.global.cleanup_empty_folders.min_age_seconds = 0;
        rules
    }

    fn write_file(path: &Path, content: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        fs::write(path, content).expect("write test file");
    }

    fn tear_down(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn acceptance_flatten_and_misc_routing() {
        let root = temp_sort_root();
        let rules = make_rules(&root);
        rules::ensure_sort_root_dirs(&rules).expect("ensure sort root dirs");

        write_file(&root.join("SomeDump/A/B/song.mp3"), b"audio");
        write_file(&root.join("SomeDump/A/readme.txt"), b"doc");
        write_file(&root.join("SomeDump/A/unknown.weird"), b"unknown");
        write_file(&root.join("SomeDump/A/noext"), b"none");

        let plan = planner::build_plan(&rules).expect("build plan");
        assert_eq!(plan.move_count, 4);

        let run = executor::execute_plan_silent(&plan).expect("execute plan");
        assert_eq!(run.moved, 4);
        assert_eq!(run.errors, 0);

        assert!(root.join("Audio/song.mp3").exists());
        assert!(root.join("Documents/readme.txt").exists());
        assert!(root.join("Misc/unknown.weird").exists());
        assert!(root.join("Misc/noext").exists());

        tear_down(&root);
    }

    #[test]
    fn acceptance_collision_rename_policy() {
        let root = temp_sort_root();
        let rules = make_rules(&root);
        rules::ensure_sort_root_dirs(&rules).expect("ensure sort root dirs");

        write_file(&root.join("Documents/report.txt"), b"existing target");
        write_file(&root.join("BatchOne/report.txt"), b"a");
        write_file(&root.join("BatchTwo/report.txt"), b"b");

        let plan = planner::build_plan(&rules).expect("build plan");
        assert_eq!(plan.move_count, 2);
        assert_eq!(plan.potential_conflicts, 2);

        let destinations: BTreeSet<_> = plan
            .moves
            .iter()
            .map(|entry| PathBuf::from(&entry.destination_path))
            .collect();

        assert!(destinations.contains(&root.join("Documents/report (1).txt")));
        assert!(destinations.contains(&root.join("Documents/report (2).txt")));

        let run = executor::execute_plan_silent(&plan).expect("execute plan");
        assert_eq!(run.errors, 0);
        assert!(root.join("Documents/report (1).txt").exists());
        assert!(root.join("Documents/report (2).txt").exists());

        tear_down(&root);
    }

    #[test]
    fn acceptance_cleanup_protects_category_and_root() {
        let root = temp_sort_root();
        let rules = make_rules(&root);
        rules::ensure_sort_root_dirs(&rules).expect("ensure sort root dirs");

        write_file(&root.join("Incoming/Nested/clip.mp4"), b"video");

        let plan = planner::build_plan(&rules).expect("build plan");
        let run = executor::execute_plan_silent(&plan).expect("execute plan");
        assert_eq!(run.errors, 0);

        let cleanup = cleanup::cleanup_empty_folders(&rules).expect("cleanup");
        assert_eq!(cleanup.errors, 0);

        assert!(root.exists());
        for folder in [
            "Documents",
            "Images",
            "Video",
            "Audio",
            "Archives",
            "Code",
            "Executables",
            "Data",
            "Misc",
        ] {
            assert!(root.join(folder).exists(), "protected folder missing: {}", folder);
        }

        assert!(!root.join("Incoming/Nested").exists());

        tear_down(&root);
    }

    #[test]
    fn acceptance_dry_run_matches_run_destinations() {
        let root = temp_sort_root();
        let rules = make_rules(&root);
        rules::ensure_sort_root_dirs(&rules).expect("ensure sort root dirs");

        write_file(&root.join("Drop/img.png"), b"img");
        write_file(&root.join("Drop/movie.mkv"), b"video");
        write_file(&root.join("Drop/archive.zip"), b"zip");

        let plan = planner::build_plan(&rules).expect("build plan");
        let expected: BTreeSet<_> = plan
            .moves
            .iter()
            .map(|entry| entry.destination_path.clone())
            .collect();

        let run = executor::execute_plan_silent(&plan).expect("execute plan");
        assert_eq!(run.errors, 0);
        assert_eq!(run.moved, plan.move_count);
        assert_eq!(run.skipped, plan.skip_count);

        let actual: BTreeSet<_> = run
            .moved_files
            .iter()
            .map(|entry| entry.destination_path.clone())
            .collect();

        assert_eq!(actual, expected);

        tear_down(&root);
    }

    #[test]
    fn acceptance_undo_last_run_best_effort() {
        let root = temp_sort_root();
        let mut rules = make_rules(&root);
        rules.global.cleanup_empty_folders.enabled = false;
        rules::ensure_sort_root_dirs(&rules).expect("ensure sort root dirs");

        write_file(&root.join("Load/invoice.txt"), b"doc");
        write_file(&root.join("Load/song.mp3"), b"audio");

        let plan = planner::build_plan(&rules).expect("build plan");
        let run = executor::execute_plan_silent(&plan).expect("execute plan");
        assert_eq!(run.errors, 0);
        assert!(run.moved >= 2);

        let journal_path = root.join("journal.jsonl");
        journal::append_run(&journal_path, &run.session_id, &run.moved_files).expect("append run");

        let conflict_source = PathBuf::from(&run.moved_files[0].source_path);
        write_file(&conflict_source, b"occupied");

        let undo = journal::undo_last_run(&journal_path).expect("undo last run");

        assert!(undo.skipped >= 1);
        assert!(undo.restored >= 1);
        assert_eq!(undo.errors, 0);

        tear_down(&root);
    }
}
