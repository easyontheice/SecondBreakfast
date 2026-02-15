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
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use watcher::{DebouncedAction, EventObserver, WatcherController, WatcherStatus};

#[derive(Clone)]
struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Debug, Clone)]
struct OriginHint {
    observed_path: PathBuf,
    original_path: PathBuf,
}

struct AppStateInner {
    rules: Mutex<Rules>,
    rules_path: PathBuf,
    journal_path: PathBuf,
    watcher: Arc<Mutex<WatcherController>>,
    pipeline_running: AtomicBool,
    undo_in_progress: AtomicBool,
    origin_hints: Mutex<Vec<OriginHint>>,
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
            undo_in_progress: AtomicBool::new(false),
            origin_hints: Mutex::new(Vec::new()),
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

struct BoolGuard<'a> {
    flag: &'a AtomicBool,
}

impl<'a> BoolGuard<'a> {
    fn set(flag: &'a AtomicBool, value: bool) -> Self {
        flag.store(value, Ordering::SeqCst);
        Self { flag }
    }
}

impl Drop for BoolGuard<'_> {
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

    let overrides = resolve_original_path_overrides(state, &result.moved_files)?;
    journal::append_run(
        &state.inner.journal_path,
        &result.session_id,
        &result.moved_files,
        &overrides,
    )?;
    clear_origin_hints(state)?;


    if should_emit_run_complete(&result) {
        let _ = app.emit("run_complete", result.clone());
    }
    Ok(result)
}

fn undo_last_run_internal(app: &AppHandle, state: &AppState) -> AppResult<journal::UndoResult> {
    let _guard = RunGuard::acquire(&state.inner.pipeline_running)?;
    let _undo_guard = BoolGuard::set(&state.inner.undo_in_progress, true);
    let watcher_was_running = state.watcher_running()?;

    if watcher_was_running {
        stop_watcher_internal(app, state)?;
    }

    let undo_result = journal::undo_last_run(&state.inner.journal_path);

    if watcher_was_running {
        std::thread::sleep(Duration::from_millis(1500));
        start_watcher_internal(app, state)?;
    }

    let result = undo_result?;
    executor::emit_log(
        app,
        "info",
        format!(
            "undo complete: restored={}, skipped={}, conflicts={}, missing={}, errors={}",
            result.restored, result.skipped, result.conflicts, result.missing, result.errors
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
        if state_clone.inner.undo_in_progress.load(Ordering::SeqCst) {
            return;
        }

        if let Err(err) = prune_origin_hints(&state_clone) {
            executor::emit_log(&app_handle, "warn", format!("prune_origin_hints failed: {}", err));
        }

        if let Err(err) = run_now_internal(&app_handle, &state_clone) {
            executor::emit_log(&app_handle, "error", format!("watcher-triggered run failed: {}", err));
        }
    });


    let hint_state = state.clone();
    let hint_sort_root = sort_root.clone();
    let observer: EventObserver = Arc::new(move |event| {
        capture_origin_hint(&hint_state, &hint_sort_root, event);
    });

    watcher::start_watcher(
        &state.inner.watcher,
        sort_root,
        Duration::from_secs(2),
        action,
        Some(observer),
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

fn clear_origin_hints(state: &AppState) -> AppResult<()> {
    state.inner.origin_hints.lock()?.clear();
    Ok(())
}

fn capture_origin_hint(state: &AppState, sort_root: &Path, event: &notify::Event) {
    if event.paths.len() < 2 {
        return;
    }

    let from = event.paths[0].clone();
    let to = event.paths[1].clone();

    let from_inside = from.starts_with(sort_root);
    let to_inside = to.starts_with(sort_root);

    let Ok(mut hints) = state.inner.origin_hints.lock() else {
        return;
    };

    if to_inside && !from_inside {
        if !from.is_absolute() || !to.is_absolute() {
            return;
        }

        let observed_key = path_key(&to);
        if let Some(existing) = hints
            .iter_mut()
            .find(|entry| path_key(&entry.observed_path) == observed_key)
        {
            existing.original_path = from;
        } else {
            hints.push(OriginHint {
                observed_path: to,
                original_path: from,
            });
        }
        return;
    }

    if from_inside && !to_inside {
        hints.retain(|entry| !entry.observed_path.starts_with(&from));
        return;
    }

    if from_inside && to_inside {
        for entry in hints.iter_mut() {
            if !entry.observed_path.starts_with(&from) {
                continue;
            }

            let Ok(relative) = entry.observed_path.strip_prefix(&from) else {
                continue;
            };

            entry.observed_path = if relative.as_os_str().is_empty() {
                to.clone()
            } else {
                to.join(relative)
            };
        }
    }
}

fn resolve_original_path_overrides(
    state: &AppState,
    moved_files: &[executor::MovedFile],
) -> AppResult<HashMap<String, String>> {
    let mut hints = state.inner.origin_hints.lock()?.clone();
    hints.sort_by(|left, right| {
        right
            .observed_path
            .components()
            .count()
            .cmp(&left.observed_path.components().count())
    });

    let mut overrides = HashMap::with_capacity(moved_files.len());

    for moved in moved_files {
        let source = PathBuf::from(&moved.source_path);
        let mut resolved = source.clone();

        if source.is_absolute() {
            for hint in &hints {
                let Ok(relative) = source.strip_prefix(&hint.observed_path) else {
                    continue;
                };

                let candidate = if relative.as_os_str().is_empty() {
                    hint.original_path.clone()
                } else {
                    hint.original_path.join(relative)
                };

                if candidate.is_absolute() {
                    resolved = candidate;
                }
                break;
            }
        }

        overrides.insert(
            moved.source_path.clone(),
            resolved.to_string_lossy().to_string(),
        );
    }

    Ok(overrides)
}

fn prune_origin_hints(state: &AppState) -> AppResult<()> {
    let mut hints = state.inner.origin_hints.lock()?;
    hints.retain(|entry| entry.observed_path.exists());
    Ok(())
}

fn path_key(path: &Path) -> String {
    let raw = path.to_string_lossy();
    if cfg!(windows) {
        raw.to_ascii_lowercase()
    } else {
        raw.to_string()
    }
}

fn apply_cleanup(result: &mut RunResult, cleanup: CleanupResult) {
    result.cleanup_trashed = cleanup.trashed;
    result.cleanup_errors = cleanup.errors;
}

fn should_emit_run_complete(result: &RunResult) -> bool {
    result.moved > 0 || result.skipped > 0 || result.errors > 0
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
    use std::collections::{BTreeSet, HashMap};
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

       let overrides = resolve_original_path_overrides(state, &run.moved_files)?;
        journal::append_run(&journal_path, &run.session_id, &run.moved_files, &overrides)
            .expect("append run");
        clear_origin_hints(state).expect("clear origin hints");





        let conflict_source = PathBuf::from(&run.moved_files[0].source_path);
        write_file(&conflict_source, b"occupied");

        let undo = journal::undo_last_run(&journal_path).expect("undo last run");

        assert!(undo.conflicts >= 1);
        assert!(undo.restored >= 1);
        assert_eq!(undo.errors, 0);

        tear_down(&root);
    }
}










