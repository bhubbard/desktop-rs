use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Failed to parse diff: {0}")]
    DiffParseError(String),

    #[error("Invalid patch hunk: {0}")]
    InvalidHunk(String),

    #[error("Invalid commit trailer: {0}")]
    InvalidTrailer(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
