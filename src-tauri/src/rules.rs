use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rules {
    pub global: GlobalRules,
    pub categories: Vec<CategoryRule>,
    pub misc: MiscRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalRules {
    pub sort_root: String,
    pub case_insensitive_ext: bool,
    pub collision_policy: CollisionPolicy,
    pub unknown_goes_to_misc: bool,
    pub no_extension_goes_to_misc: bool,
    pub min_file_age_seconds: u64,
    pub cleanup_empty_folders: CleanupRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollisionPolicy {
    Rename,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupRules {
    pub enabled: bool,
    pub min_age_seconds: u64,
    pub mode: CleanupMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CleanupMode {
    Trash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRule {
    pub id: String,
    pub name: String,
    pub target_subfolder: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiscRule {
    pub name: String,
    pub target_subfolder: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn suggested_sort_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Sort")
}

pub fn config_dir() -> AppResult<PathBuf> {
    let base = dirs::config_dir().ok_or(AppError::ConfigDirUnavailable)?;
    let app_dir = base.join("sort-root");
    fs::create_dir_all(&app_dir)?;
    Ok(app_dir)
}

pub fn rules_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("rules.json"))
}

pub fn journal_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("journal.jsonl"))
}

pub fn default_rules() -> Rules {
    Rules {
        global: GlobalRules {
            sort_root: suggested_sort_root().to_string_lossy().to_string(),
            case_insensitive_ext: true,
            collision_policy: CollisionPolicy::Rename,
            unknown_goes_to_misc: true,
            no_extension_goes_to_misc: true,
            min_file_age_seconds: 10,
            cleanup_empty_folders: CleanupRules {
                enabled: true,
                min_age_seconds: 60,
                mode: CleanupMode::Trash,
            },
        },
        categories: vec![
            CategoryRule {
                id: "documents".to_string(),
                name: "Documents".to_string(),
                target_subfolder: "Documents".to_string(),
                extensions: [
                    "doc", "docx", "rtf", "txt", "md", "pdf", "odt", "xls", "xlsx", "ods", "csv",
                    "ppt", "pptx", "epub",
                ]
                .iter()
                .map(|x| x.to_string())
                .collect(),
            },
            CategoryRule {
                id: "images".to_string(),
                name: "Images".to_string(),
                target_subfolder: "Images".to_string(),
                extensions: ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tif", "tiff", "svg", "ico", "psd"]
                    .iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
            CategoryRule {
                id: "video".to_string(),
                name: "Video".to_string(),
                target_subfolder: "Video".to_string(),
                extensions: ["mp4", "mkv", "mov", "avi", "wmv", "webm", "m4v"]
                    .iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
            CategoryRule {
                id: "audio".to_string(),
                name: "Audio".to_string(),
                target_subfolder: "Audio".to_string(),
                extensions: ["mp3", "wav", "flac", "aac", "m4a", "ogg"]
                    .iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
            CategoryRule {
                id: "archives".to_string(),
                name: "Archives".to_string(),
                target_subfolder: "Archives".to_string(),
                extensions: ["zip", "rar", "7z", "tar", "gz", "tgz", "bz2", "iso"]
                    .iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
            CategoryRule {
                id: "code".to_string(),
                name: "Code".to_string(),
                target_subfolder: "Code".to_string(),
                extensions: [
                    "py", "js", "ts", "html", "htm", "css", "c", "cpp", "h", "hpp", "cs", "java", "sh", "bat",
                    "ps1", "json", "yaml", "yml", "xml",
                ]
                .iter()
                .map(|x| x.to_string())
                .collect(),
            },
            CategoryRule {
                id: "executables".to_string(),
                name: "Executables".to_string(),
                target_subfolder: "Executables".to_string(),
                extensions: ["exe", "msi", "deb", "rpm", "app", "apk", "jar"]
                    .iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
            CategoryRule {
                id: "data".to_string(),
                name: "Data".to_string(),
                target_subfolder: "Data".to_string(),
                extensions: ["db", "sqlite", "sql", "parquet"]
                    .iter()
                    .map(|x| x.to_string())
                    .collect(),
            },
        ],
        misc: MiscRule {
            name: "Misc".to_string(),
            target_subfolder: "Misc".to_string(),
        },
    }
}

pub fn load_or_create_rules(path: &Path) -> AppResult<Rules> {
    if path.exists() {
        let content = fs::read_to_string(path)?;
        let parsed: Rules = serde_json::from_str(&content)?;
        let validation = validate_rules(&parsed);
        if !validation.valid {
            return Err(AppError::Validation(validation.errors.join("; ")));
        }
        Ok(parsed)
    } else {
        let rules = default_rules();
        save_rules(path, &rules)?;
        Ok(rules)
    }
}

pub fn save_rules(path: &Path, rules: &Rules) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_string_pretty(rules)?;
    fs::write(path, payload)?;
    Ok(())
}

pub fn validate_rules(rules: &Rules) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if rules.global.sort_root.trim().is_empty() {
        errors.push("sortRoot cannot be empty".to_string());
    }

    if rules.categories.is_empty() {
        errors.push("at least one category is required".to_string());
    }

    let mut seen_ext = HashMap::new();
    for category in &rules.categories {
        if category.target_subfolder.trim().is_empty() {
            errors.push(format!("category '{}' has empty targetSubfolder", category.name));
        }

        for ext in &category.extensions {
            let norm = normalize_extension(ext, rules.global.case_insensitive_ext);
            if norm.is_empty() {
                warnings.push(format!("category '{}' includes empty extension", category.name));
                continue;
            }
            if let Some(prev) = seen_ext.insert(norm.clone(), category.name.clone()) {
                warnings.push(format!(
                    "extension '{}' is defined in both '{}' and '{}'; first match wins",
                    norm, prev, category.name
                ));
            }
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

pub fn normalize_extension(ext: &str, case_insensitive: bool) -> String {
    let ext = ext.trim().trim_start_matches('.');
    if case_insensitive {
        ext.to_ascii_lowercase()
    } else {
        ext.to_string()
    }
}

pub fn extension_lookup(rules: &Rules) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for category in &rules.categories {
        for ext in &category.extensions {
            let key = normalize_extension(ext, rules.global.case_insensitive_ext);
            if key.is_empty() {
                continue;
            }
            map.entry(key)
                .or_insert_with(|| category.target_subfolder.clone());
        }
    }
    map
}

pub fn protected_top_level_folders(rules: &Rules) -> HashSet<String> {
    let mut set = HashSet::new();
    for category in &rules.categories {
        set.insert(category.target_subfolder.clone());
    }
    set.insert(rules.misc.target_subfolder.clone());
    set
}

pub fn ensure_sort_root_dirs(rules: &Rules) -> AppResult<()> {
    let sort_root = PathBuf::from(&rules.global.sort_root);
    fs::create_dir_all(&sort_root)?;

    let protected = protected_top_level_folders(rules);
    for folder in protected {
        fs::create_dir_all(sort_root.join(folder))?;
    }
    Ok(())
}
