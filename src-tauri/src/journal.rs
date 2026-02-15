use crate::errors::AppResult;
use crate::executor::MovedFile;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalRun {
    #[serde(alias = "run_id")]
    pub session_id: String,
    pub created_at: String,
    pub moves: Vec<JournalMove>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalMove {
    #[serde(default)]
    pub run_id: String,
    #[serde(alias = "source_path")]
    pub original_path: String,
    #[serde(alias = "destination_path")]
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
        if let Ok(run) = serde_json::from_str::<JournalRun>(line) {
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
