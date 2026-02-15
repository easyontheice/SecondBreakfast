use std::io;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("watcher error: {0}")]
    Notify(#[from] notify::Error),
    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
    #[error("config directory is unavailable")]
    ConfigDirUnavailable,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("state error: {0}")]
    State(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::State(err.to_string())
    }
}
