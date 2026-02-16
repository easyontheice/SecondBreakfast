use crate::errors::AppResult;
use crate::executor::MovedFile;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalRun {
    #[serde(rename = "session_id", alias = "sessionId", alias = "run_id", alias = "runId")]
    pub session_id: String,
    #[serde(rename = "created_at", alias = "createdAt", default = "default_timestamp")]
    pub created_at: String,
    pub moves: Vec<JournalMove>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalMove {
    #[serde(rename = "run_id", alias = "runId", default)]
    pub run_id: String,
    #[serde(
        rename = "original_path",
        alias = "originalPath",
        alias = "source_path",
        alias = "sourcePath",
        default
    )]
    pub original_path: String,
    #[serde(
        rename = "new_path",
        alias = "newPath",
        alias = "destination_path",
        alias = "destinationPath",
        default
    )]
    pub new_path: String,
    #[serde(default = "default_timestamp")]
    pub timestamp: String,
    #[serde(default = "default_moved_status")]
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDetail {
    pub source_path: String,
    pub destination_path: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoResult {
    pub session_id: Option<String>,
    pub restored: u64,
    pub skipped: u64,
    pub conflicts: u64,
    pub missing: u64,
    pub errors: u64,
    pub details: Vec<UndoDetail>,
}

pub fn append_run(
    path: &Path,
    session_id: &str,
    moved_files: &[MovedFile],
    original_path_overrides: &HashMap<String, String>,
) -> AppResult<()> {
    if moved_files.is_empty() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let run = JournalRun {
        session_id: session_id.to_string(),
        created_at: Utc::now().to_rfc3339(),
        moves: moved_files
            .iter()
            .map(|item| JournalMove {
                run_id: session_id.to_string(),
                original_path: original_path_overrides
                    .get(&item.source_path)
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
                    .unwrap_or_else(|| item.source_path.clone()),
                new_path: item.destination_path.clone(),
                timestamp: Utc::now().to_rfc3339(),
                status: default_moved_status(),
            })
            .collect(),
    };

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(&run)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

pub fn load_last_run(path: &Path) -> AppResult<Option<JournalRun>> {
    if !path.exists() {
        return Ok(None);
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut last = None;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(mut run) = serde_json::from_str::<JournalRun>(line) {
            for movement in &mut run.moves {
                if movement.run_id.is_empty() {
                    movement.run_id = run.session_id.clone();
                }
                if movement.status.trim().is_empty() {
                    movement.status = default_moved_status();
                }
            }
            last = Some(run);
        }
    }

    Ok(last)
}

/// Convert an absolute path into a safe *relative* path that preserves structure.
///
/// - Windows: `C:\Users\Me\file.txt` -> `C/Users/Me/file.txt`
/// - Unix: `/Users/Me/file.txt` -> `Users/Me/file.txt`
fn absolute_to_safe_relative(original: &Path) -> Option<PathBuf> {
    if !original.is_absolute() {
        return None;
    }

    // Best-effort string-based conversion to handle Windows drive letters cleanly.
    let s = original.to_string_lossy();

    // Windows drive letter pattern like "C:\..."
    if s.len() >= 2 && s.as_bytes()[1] == b':' {
        let drive = s.chars().next()?.to_string(); // "C"
        let rest = s[2..].trim_start_matches(['\\', '/']).replace('\\', "/");
        let rel = PathBuf::from(drive).join(rest);

        // Safety: prevent traversal
        if rel.components().any(|c| matches!(c, Component::ParentDir)) {
            return None;
        }
        return Some(rel);
    }

    // Unix absolute: trim leading "/"
    let rest = s.trim_start_matches('/');
    let rel = PathBuf::from(rest);

    if rel.components().any(|c| matches!(c, Component::ParentDir)) {
        return None;
    }

    Some(rel)
}

/// Undo restores into `<sort_root>/Restored/<session_id>/...`
/// preserving the original absolute path structure as a relative tree.
pub fn undo_last_run(path: &Path, sort_root: &Path) -> AppResult<UndoResult> {
    let Some(last) = load_last_run(path)? else {
        return Ok(UndoResult {
            session_id: None,
            restored: 0,
            skipped: 0,
            conflicts: 0,
            missing: 0,
            errors: 0,
            details: Vec::new(),
        });
    };

    // Deterministic restore base: <sort_root>/Restored/<session_id>
    let restored_base = sort_root.join("Restored").join(&last.session_id);
    fs::create_dir_all(&restored_base)?;

    let mut result = UndoResult {
        session_id: Some(last.session_id.clone()),
        restored: 0,
        skipped: 0,
        conflicts: 0,
        missing: 0,
        errors: 0,
        details: Vec::new(),
    };

    for movement in last.moves.iter().rev() {
        if movement.status != "moved" {
            result.skipped += 1;
            result.details.push(UndoDetail {
                source_path: movement.original_path.clone(),
                destination_path: movement.new_path.clone(),
                status: "skipped".to_string(),
                message: format!("journal status '{}' is not undoable", movement.status),
            });
            continue;
        }

        if movement.original_path.trim().is_empty() || movement.new_path.trim().is_empty() {
            result.skipped += 1;
            result.details.push(UndoDetail {
                source_path: movement.original_path.clone(),
                destination_path: movement.new_path.clone(),
                status: "skipped".to_string(),
                message: "journal entry missing original_path or dest_path".to_string(),
            });
            continue;
        }

        let original = PathBuf::from(&movement.original_path);
        let current = PathBuf::from(&movement.new_path);

        if !original.is_absolute() || !current.is_absolute() {
            result.skipped += 1;
            result.details.push(UndoDetail {
                source_path: movement.original_path.clone(),
                destination_path: movement.new_path.clone(),
                status: "skipped".to_string(),
                message: "journal paths must be absolute".to_string(),
            });
            continue;
        }

        if !current.exists() {
            result.missing += 1;
            result.details.push(UndoDetail {
                source_path: movement.original_path.clone(),
                destination_path: movement.new_path.clone(),
                status: "missing".to_string(),
                message: "destination no longer exists".to_string(),
            });
            continue;
        }

        // Convert original absolute path into a safe relative tree under Restored/<session_id>.
        let Some(rel) = absolute_to_safe_relative(&original) else {
            result.skipped += 1;
            result.details.push(UndoDetail {
                source_path: movement.original_path.clone(),
                destination_path: movement.new_path.clone(),
                status: "skipped".to_string(),
                message: "could not derive safe relative restore path".to_string(),
            });
            continue;
        };

        let mut target = restored_base.join(rel);

        let mut conflict_target = None;
        if target.exists() {
            let next = resolve_restored_conflict_path(&target);
            conflict_target = Some(next.clone());
            target = next;
            result.conflicts += 1;
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        match move_file(&current, &target) {
            Ok(()) => {
                result.restored += 1;

                let (status, message) = if let Some(conflict) = conflict_target {
                    (
                        "conflict".to_string(),
                        format!("restored to conflict path {}", conflict.to_string_lossy()),
                    )
                } else {
                    (
                        "restored".to_string(),
                        format!("restored under {}", restored_base.to_string_lossy()),
                    )
                };

                result.details.push(UndoDetail {
                    source_path: movement.original_path.clone(),
                    destination_path: movement.new_path.clone(),
                    status,
                    message,
                });
            }
            Err(err) => {
                result.errors += 1;
                result.details.push(UndoDetail {
                    source_path: movement.original_path.clone(),
                    destination_path: movement.new_path.clone(),
                    status: "error".to_string(),
                    message: err.to_string(),
                });
            }
        }
    }

    Ok(result)
}

fn move_file(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    std::fs::rename(src, dest).or_else(|_| {
        std::fs::copy(src, dest)?;
        std::fs::remove_file(src)?;
        Ok(())
    })
}

fn resolve_restored_conflict_path(original: &Path) -> PathBuf {
    let parent = original
        .parent()
        .map(|value| value.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let stem = original
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let ext = original
        .extension()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut idx = 1_u64;
    loop {
        let file_name = if ext.is_empty() {
            format!("{} (restored {})", stem, idx)
        } else {
            format!("{} (restored {}).{}", stem, idx, ext)
        };

        let candidate = parent.join(file_name);
        if !candidate.exists() {
            return candidate;
        }

        idx += 1;
    }
}

fn default_timestamp() -> String {
    Utc::now().to_rfc3339()
}

fn default_moved_status() -> String {
    "moved".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!("sortroot-journal-{}", Uuid::new_v4()))
    }

    #[test]
    fn undo_supports_legacy_camelcase_paths_and_restores_under_restored_folder() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("create temp root");

        // "Original" path in journal: absolute
        let source = root.join("Drop").join("Nested").join("invoice.txt");
        // "Current" path in journal: absolute
        let destination = root.join("Documents").join("invoice.txt");

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).expect("create destination parent");
        }
        fs::write(&destination, b"payload").expect("write destination file");

        let journal_path = root.join("journal.jsonl");
        let legacy = serde_json::json!({
            "sessionId": "legacy-run",
            "createdAt": Utc::now().to_rfc3339(),
            "moves": [
                {
                    "sourcePath": source.to_string_lossy(),
                    "destinationPath": destination.to_string_lossy(),
                    "timestamp": Utc::now().to_rfc3339()
                }
            ]
        });
        fs::write(&journal_path, format!("{}\n", legacy)).expect("write legacy journal");

        let result = undo_last_run(&journal_path, &root).expect("undo run");

        assert_eq!(result.restored, 1);
        assert!(!destination.exists());

        // Restored files should exist under root/Restored/<session_id>/... (we only assert the parent exists)
        let restored_dir = root.join("Restored").join("legacy-run");
        assert!(restored_dir.exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn append_run_writes_required_paths() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("create temp root");

        let journal_path = root.join("journal.jsonl");
        let moved = vec![MovedFile {
            source_path: root.join("Drop").join("a.txt").to_string_lossy().to_string(),
            destination_path: root
                .join("Documents")
                .join("a.txt")
                .to_string_lossy()
                .to_string(),
            category: "Documents".to_string(),
            collision_renamed: false,
        }];

        let overrides = HashMap::new();
        append_run(&journal_path, "run-1", &moved, &overrides).expect("append journal");

        let line = fs::read_to_string(&journal_path).expect("read journal");
        assert!(line.contains("\"original_path\""));
        assert!(line.contains("\"new_path\""));
        assert!(line.contains("\"run_id\""));
        assert!(line.contains("\"status\""));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn undo_uses_conflict_name_when_target_is_occupied_under_restored_folder() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("create temp root");

        let original = root.join("Dump").join("Nested").join("clip.mp4");
        let destination = root.join("Video").join("clip.mp4");

        fs::create_dir_all(destination.parent().expect("dest parent")).expect("dest parent create");
        fs::create_dir_all(original.parent().expect("orig parent")).expect("orig parent create");
        fs::write(&destination, b"restored payload").expect("write destination file");

        // Pre-create the would-be restored target under Restored/<session_id>/... to force a conflict.
        let restored_root = root.join("Restored").join("run-2");
        let rel = absolute_to_safe_relative(&original).expect("safe relative");
        let would_be_target = restored_root.join(rel);

        if let Some(parent) = would_be_target.parent() {
            fs::create_dir_all(parent).expect("create restored parent");
        }
        fs::write(&would_be_target, b"occupied").expect("occupy would-be target");

        let journal_path = root.join("journal.jsonl");
        let entry = serde_json::json!({
            "sessionId": "run-2",
            "createdAt": Utc::now().to_rfc3339(),
            "moves": [
                {
                    "sourcePath": original.to_string_lossy(),
                    "destinationPath": destination.to_string_lossy(),
                    "status": "moved",
                    "timestamp": Utc::now().to_rfc3339()
                }
            ]
        });
        fs::write(&journal_path, format!("{}\n", entry)).expect("write journal");

        let result = undo_last_run(&journal_path, &root).expect("undo run");

        assert_eq!(result.errors, 0);
        assert_eq!(result.restored, 1);
        assert!(result.conflicts >= 1);
        assert!(!destination.exists());
        assert!(root.join("Restored").join("run-2").exists());

        let _ = fs::remove_dir_all(&root);
    }
}
