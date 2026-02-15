use crate::errors::AppResult;
use crate::rules::{protected_top_level_folders, Rules};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub trashed: u64,
    pub skipped: u64,
    pub errors: u64,
    pub skipped_paths: Vec<String>,
}

pub fn cleanup_empty_folders(rules: &Rules) -> AppResult<CleanupResult> {
    if !rules.global.cleanup_empty_folders.enabled {
        return Ok(CleanupResult {
            trashed: 0,
            skipped: 0,
            errors: 0,
            skipped_paths: Vec::new(),
        });
    }

    let root = Path::new(&rules.global.sort_root);
    let protected = protected_top_level_folders(rules);
    let mut result = CleanupResult {
        trashed: 0,
        skipped: 0,
        errors: 0,
        skipped_paths: Vec::new(),
    };

    for entry in WalkDir::new(root)
        .min_depth(1)
        .contents_first(true)
        .into_iter()
    {
        let entry = match entry {
            Ok(value) => value,
            Err(err) => {
                result.errors += 1;
                result.skipped_paths.push(err.to_string());
                continue;
            }
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        if is_protected(path, root, &protected) {
            result.skipped += 1;
            continue;
        }

        match fs::read_dir(path) {
            Ok(mut dir_entries) => {
                if dir_entries.next().is_none() {
                    match trash::delete(path) {
                        Ok(()) => result.trashed += 1,
                        Err(err) => {
                            result.errors += 1;
                            result
                                .skipped_paths
                                .push(format!("{}: {}", path.to_string_lossy(), err));
                        }
                    }
                }
            }
            Err(err) => {
                result.errors += 1;
                result
                    .skipped_paths
                    .push(format!("{}: {}", path.to_string_lossy(), err));
            }
        }
    }

    Ok(result)
}

fn is_protected(path: &Path, root: &Path, protected: &std::collections::HashSet<String>) -> bool {
    if path == root {
        return true;
    }

    let Ok(relative) = path.strip_prefix(root) else {
        return true;
    };

    let Some(first) = relative.iter().next() else {
        return true;
    };

    protected.contains(&first.to_string_lossy().to_string())
}
