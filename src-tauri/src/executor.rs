use crate::errors::AppResult;
use crate::planner::{PlanEntry, PlanPreview, PlanSkip};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovedFile {
    pub source_path: String,
    pub destination_path: String,
    pub category: String,
    pub collision_renamed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    pub session_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub moved: u64,
    pub skipped: u64,
    pub errors: u64,
    pub moved_files: Vec<MovedFile>,
    pub skips: Vec<PlanSkip>,
    pub error_details: Vec<PlanSkip>,
    pub cleanup_trashed: u64,
    pub cleanup_errors: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunProgressEvent {
    moved: u64,
    skipped: u64,
    errors: u64,
    current_path: Option<String>,
    dest_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunLogEvent {
    level: String,
    message: String,
}

pub fn execute_plan(app: &AppHandle, plan: &PlanPreview) -> AppResult<RunResult> {
    execute_plan_impl(Some(app), plan)
}

pub fn execute_plan_silent(plan: &PlanPreview) -> AppResult<RunResult> {
    execute_plan_impl(None, plan)
}

fn execute_plan_impl(app: Option<&AppHandle>, plan: &PlanPreview) -> AppResult<RunResult> {
    let started_at = Utc::now().to_rfc3339();
    let mut moved = 0_u64;
    let mut errors = 0_u64;
    let mut moved_files = Vec::new();
    let mut error_details = Vec::new();

    emit_log_opt(app, "info", format!("run started: {} planned moves", plan.move_count));

    for item in &plan.moves {
        match move_entry(item) {
            Ok(()) => {
                moved += 1;
                moved_files.push(MovedFile {
                    source_path: item.source_path.clone(),
                    destination_path: item.destination_path.clone(),
                    category: item.category.clone(),
                    collision_renamed: item.collision_renamed,
                });

                emit_progress_opt(
                    app,
                    moved,
                    plan.skip_count,
                    errors,
                    Some(item.source_path.clone()),
                    Some(item.destination_path.clone()),
                );
            }
            Err(err) => {
                errors += 1;
                error_details.push(PlanSkip {
                    path: item.source_path.clone(),
                    reason: err.to_string(),
                });
                emit_log_opt(
                    app,
                    "error",
                    format!("failed moving '{}' => {}", item.source_path, err),
                );
                emit_progress_opt(
                    app,
                    moved,
                    plan.skip_count,
                    errors,
                    Some(item.source_path.clone()),
                    Some(item.destination_path.clone()),
                );
            }
        }
    }

    let finished_at = Utc::now().to_rfc3339();
    emit_log_opt(
        app,
        "info",
        format!(
            "run complete: moved={}, skipped={}, errors={}",
            moved,
            plan.skip_count,
            errors
        ),
    );

    Ok(RunResult {
        session_id: plan.session_id.clone(),
        started_at,
        finished_at,
        moved,
        skipped: plan.skip_count,
        errors,
        moved_files,
        skips: plan.skips.clone(),
        error_details,
        cleanup_trashed: 0,
        cleanup_errors: 0,
    })
}

fn move_entry(entry: &PlanEntry) -> AppResult<()> {
    let src = Path::new(&entry.source_path);
    let dest = Path::new(&entry.destination_path);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::rename(src, dest) {
        Ok(()) => Ok(()),
        Err(_) => {
            fs::copy(src, dest)?;
            fs::remove_file(src)?;
            Ok(())
        }
    }
}

fn emit_progress_opt(
    app: Option<&AppHandle>,
    moved: u64,
    skipped: u64,
    errors: u64,
    current_path: Option<String>,
    dest_path: Option<String>,
) {
    let Some(app) = app else {
        return;
    };

    let _ = app.emit(
        "run_progress",
        RunProgressEvent {
            moved,
            skipped,
            errors,
            current_path,
            dest_path,
        },
    );
}

fn emit_log_opt(app: Option<&AppHandle>, level: &str, message: impl Into<String>) {
    let Some(app) = app else {
        return;
    };

    let _ = app.emit(
        "run_log",
        RunLogEvent {
            level: level.to_string(),
            message: message.into(),
        },
    );
}

pub fn emit_log(app: &AppHandle, level: &str, message: impl Into<String>) {
    emit_log_opt(Some(app), level, message);
}
