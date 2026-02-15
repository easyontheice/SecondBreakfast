use crate::errors::AppResult;
use crate::executor::MovedFile;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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
        alias = "sourcePath"
    )]
    pub original_path: String,
    #[serde(
        rename = "new_path",
        alias = "newPath",
        alias = "destination_path",
        alias = "destinationPath"
    )]
    pub new_path: String,
    #[serde(default = "default_timestamp")]
    pub timestamp: String,
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
    pub errors: u64,
    pub details: Vec<UndoDetail>,
}

pub fn append_run(path: &Path, session_id: &str, moved_files: &[MovedFile]) -> AppResult<()> {
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
                original_path: item.source_path.clone(),
                new_path: item.destination_path.clone(),
                timestamp: Utc::now().to_rfc3339(),
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
            }
            last = Some(run);
        }
    }

    Ok(last)
}

pub fn undo_last_run(path: &Path) -> AppResult<UndoResult> {
    let Some(last) = load_last_run(path)? else {
        return Ok(UndoResult {
            session_id: None,
            restored: 0,
            skipped: 0,
            errors: 0,
            details: Vec::new(),
        });
    };

    let mut result = UndoResult {
        session_id: Some(last.session_id.clone()),
        restored: 0,
        skipped: 0,
        errors: 0,
        details: Vec::new(),
    };

    for movement in last.moves.iter().rev() {
        let original = Path::new(&movement.original_path);
        let current = Path::new(&movement.new_path);

        if !current.exists() {
            result.skipped += 1;
            result.details.push(UndoDetail {
                source_path: movement.original_path.clone(),
                destination_path: movement.new_path.clone(),
                status: "skipped".to_string(),
                message: "destination no longer exists".to_string(),
            });
            continue;
        }

        if original.exists() {
            result.skipped += 1;
            result.details.push(UndoDetail {
                source_path: movement.original_path.clone(),
                destination_path: movement.new_path.clone(),
                status: "skipped".to_string(),
                message: "source path already occupied".to_string(),
            });
            continue;
        }

        if let Some(parent) = original.parent() {
            fs::create_dir_all(parent)?;
        }

        let move_back: Result<(), std::io::Error> =
            std::fs::rename(current, original).or_else(|_| {
                std::fs::copy(current, original)?;
                std::fs::remove_file(current)?;
                Ok(())
            });

        match move_back {
            Ok(()) => {
                result.restored += 1;
                result.details.push(UndoDetail {
                    source_path: movement.original_path.clone(),
                    destination_path: movement.new_path.clone(),
                    status: "restored".to_string(),
                    message: "moved back".to_string(),
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

fn default_timestamp() -> String {
    Utc::now().to_rfc3339()
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
    fn undo_supports_legacy_camelcase_paths() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("create temp root");

        let source = root.join("Drop").join("Nested").join("invoice.txt");
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

        let result = undo_last_run(&journal_path).expect("undo run");

        assert_eq!(result.restored, 1);
        assert!(source.exists());
        assert!(!destination.exists());

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

        append_run(&journal_path, "run-1", &moved).expect("append journal");

        let line = fs::read_to_string(&journal_path).expect("read journal");
        assert!(line.contains("\"original_path\""));
        assert!(line.contains("\"new_path\""));
        assert!(line.contains("\"run_id\""));

        let _ = fs::remove_dir_all(&root);
    }
}
