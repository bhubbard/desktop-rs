use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("GitHub API error: status {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error(
        "No GitHub authentication token found (set GITHUB_TOKEN or login with `gh auth login`)"
    )]
    MissingToken,

    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, GitHubError>;
