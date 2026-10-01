pub mod commit;
pub mod diff;
pub mod error;
pub mod models;

pub use commit::{extract_co_authors, format_commit_message};
pub use diff::{parse_diff, Diff, DiffHunk, DiffLine, DiffLineType};
pub use error::{CoreError, Result};
pub use models::{
    Author, Branch, Commit, FileChange, FileStatusType, GitHubUser, Issue, PullRequest,
    RepositoryStatus, StashEntry,
};
