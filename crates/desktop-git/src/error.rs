use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git command failed: {command}\nExit code: {exit_code:?}\nError output: {stderr}")]
    CommandFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },

    #[error("Path '{0}' is not a valid git repository")]
    NotAGitRepository(PathBuf),

    #[error("Core error: {0}")]
    Core(#[from] desktop_core::CoreError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Operation error: {0}")]
    Operation(String),
}

pub type Result<T> = std::result::Result<T, GitError>;
