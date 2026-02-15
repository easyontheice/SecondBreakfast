use crate::errors::AppResult;
use crate::rules::{extension_lookup, normalize_extension, protected_top_level_folders, Rules};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanEntry {
    pub source_path: String,
    pub destination_path: String,
    pub category: String,
    pub collision_renamed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSkip {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanGroup {
    pub category: String,
    pub count: usize,
    pub entries: Vec<PlanEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanPreview {
    pub session_id: String,
    pub generated_at: String,
    pub total_candidates: u64,
    pub move_count: u64,
    pub skip_count: u64,
    pub error_count: u64,
    pub potential_conflicts: u64,
    pub moves: Vec<PlanEntry>,
    pub skips: Vec<PlanSkip>,
    pub grouped: Vec<PlanGroup>,
}

enum Classification {
    Target(String),
    Skip(String),
}

pub fn build_plan(rules: &Rules) -> AppResult<PlanPreview> {
    let sort_root = PathBuf::from(&rules.global.sort_root);
    let ext_map = extension_lookup(rules);
    let protected = protected_top_level_folders(rules);

    let mut total_candidates = 0_u64;
    let mut errors = 0_u64;
    let mut potential_conflicts = 0_u64;
    let mut planned = Vec::new();
    let mut skips = Vec::new();
    let mut reserved_destinations = HashSet::new();

    for entry in WalkDir::new(&sort_root).min_depth(1).into_iter() {
        let entry = match entry {
            Ok(value) => value,
            Err(err) => {
                errors += 1;
                skips.push(PlanSkip {
                    path: sort_root.to_string_lossy().to_string(),
                    reason: err.to_string(),
                });
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if is_inside_protected(path, &sort_root, &protected) {
            continue;
        }

        total_candidates += 1;

        if !is_old_enough(path, rules.global.min_file_age_seconds) {
            skips.push(PlanSkip {
                path: path.to_string_lossy().to_string(),
                reason: format!(
                    "file is younger than minFileAgeSeconds ({})",
                    rules.global.min_file_age_seconds
                ),
            });
            continue;
        }

        let target_subfolder = match classify_target(path, rules, &ext_map) {
            Classification::Target(target) => target,
            Classification::Skip(reason) => {
                skips.push(PlanSkip {
                    path: path.to_string_lossy().to_string(),
                    reason,
                });
                continue;
            }
        };

        let Some(file_name) = path.file_name() else {
            skips.push(PlanSkip {
                path: path.to_string_lossy().to_string(),
                reason: "could not determine file name".to_string(),
            });
            continue;
        };

        let dest_dir = sort_root.join(&target_subfolder);
        let candidate = dest_dir.join(file_name);
        let (dest_path, renamed) = resolve_destination(candidate, &mut reserved_destinations);

        if renamed {
            potential_conflicts += 1;
        }

        planned.push(PlanEntry {
            source_path: path.to_string_lossy().to_string(),
            destination_path: dest_path.to_string_lossy().to_string(),
            category: target_subfolder,
            collision_renamed: renamed,
        });
    }

    let mut grouped_map: HashMap<String, Vec<PlanEntry>> = HashMap::new();
    for entry in &planned {
        grouped_map
            .entry(entry.category.clone())
            .or_default()
            .push(entry.clone());
    }

    let mut grouped: Vec<PlanGroup> = grouped_map
        .into_iter()
        .map(|(category, entries)| PlanGroup {
            count: entries.len(),
            category,
            entries,
        })
        .collect();
    grouped.sort_by(|a, b| a.category.cmp(&b.category));

    Ok(PlanPreview {
        session_id: Uuid::new_v4().to_string(),
        generated_at: Utc::now().to_rfc3339(),
        total_candidates,
        move_count: planned.len() as u64,
        skip_count: skips.len() as u64,
        error_count: errors,
        potential_conflicts,
        moves: planned,
        skips,
        grouped,
    })
}

fn classify_target(path: &Path, rules: &Rules, ext_map: &HashMap<String, String>) -> Classification {
    let ext = path
        .extension()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_default();

    if ext.is_empty() {
        return if rules.global.no_extension_goes_to_misc {
            Classification::Target(rules.misc.target_subfolder.clone())
        } else {
            Classification::Skip("no extension and noExtensionGoesToMisc=false".to_string())
        };
    }

    let key = normalize_extension(&ext, rules.global.case_insensitive_ext);
    if let Some(target) = ext_map.get(&key) {
        return Classification::Target(target.clone());
    }

    if rules.global.unknown_goes_to_misc {
        Classification::Target(rules.misc.target_subfolder.clone())
    } else {
        Classification::Skip(format!("unknown extension '.{}' and unknownGoesToMisc=false", key))
    }
}

fn resolve_destination(candidate: PathBuf, reserved: &mut HashSet<PathBuf>) -> (PathBuf, bool) {
    if !candidate.exists() && !reserved.contains(&candidate) {
        reserved.insert(candidate.clone());
        return (candidate, false);
    }

    let parent = candidate
        .parent()
        .map(|x| x.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let stem = candidate
        .file_stem()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let ext = candidate
        .extension()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut idx = 1;
    loop {
        let file_name = if ext.is_empty() {
            format!("{} ({})", stem, idx)
        } else {
            format!("{} ({}).{}", stem, idx, ext)
        };
        let next = parent.join(file_name);
        if !next.exists() && !reserved.contains(&next) {
            reserved.insert(next.clone());
            return (next, true);
        }
        idx += 1;
    }
}

fn is_old_enough(path: &Path, min_age_seconds: u64) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    let Ok(age) = modified.elapsed() else {
        return false;
    };
    age.as_secs() >= min_age_seconds
}

fn is_inside_protected(path: &Path, root: &Path, protected: &HashSet<String>) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };

    let Some(first) = relative.iter().next() else {
        return false;
    };

    protected.contains(&first.to_string_lossy().to_string())
}
