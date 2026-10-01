use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatusType {
    Unmodified,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Conflicted,
}

impl FileStatusType {
    pub fn from_porcelain_code(c: char) -> Self {
        match c {
            '.' => Self::Unmodified,
            'M' => Self::Modified,
            'A' => Self::Added,
            'D' => Self::Deleted,
            'R' => Self::Renamed,
            'C' => Self::Copied,
            '?' => Self::Untracked,
            'U' => Self::Conflicted,
            _ => Self::Modified,
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Unmodified => " ",
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Renamed => "R",
            Self::Copied => "C",
            Self::Untracked => "?",
            Self::Conflicted => "!",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub old_path: Option<String>,
    pub staged_status: FileStatusType,
    pub unstaged_status: FileStatusType,
}

impl FileChange {
    pub fn new(path: impl Into<String>, staged: FileStatusType, unstaged: FileStatusType) -> Self {
        Self {
            path: path.into(),
            old_path: None,
            staged_status: staged,
            unstaged_status: unstaged,
        }
    }

    pub fn with_old_path(mut self, old_path: impl Into<String>) -> Self {
        self.old_path = Some(old_path.into());
        self
    }

    pub fn is_staged(&self) -> bool {
        self.staged_status != FileStatusType::Unmodified
            && self.staged_status != FileStatusType::Untracked
    }

    pub fn is_unstaged(&self) -> bool {
        self.unstaged_status != FileStatusType::Unmodified
            || self.staged_status == FileStatusType::Untracked
    }

    pub fn is_partially_staged(&self) -> bool {
        self.is_staged() && self.is_unstaged()
    }

    pub fn is_untracked(&self) -> bool {
        self.staged_status == FileStatusType::Untracked
            || self.unstaged_status == FileStatusType::Untracked
    }

    pub fn is_conflicted(&self) -> bool {
        self.staged_status == FileStatusType::Conflicted
            || self.unstaged_status == FileStatusType::Conflicted
    }

    pub fn summary_status(&self) -> FileStatusType {
        if self.is_conflicted() {
            FileStatusType::Conflicted
        } else if self.is_untracked() {
            FileStatusType::Untracked
        } else if self.is_staged() {
            self.staged_status
        } else {
            self.unstaged_status
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryStatus {
    pub branch: String,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub files: Vec<FileChange>,
}

impl RepositoryStatus {
    pub fn is_clean(&self) -> bool {
        self.files.is_empty()
    }

    pub fn total_changes(&self) -> usize {
        self.files.len()
    }

    pub fn staged_files(&self) -> Vec<&FileChange> {
        self.files.iter().filter(|f| f.is_staged()).collect()
    }

    pub fn unstaged_files(&self) -> Vec<&FileChange> {
        self.files.iter().filter(|f| f.is_unstaged()).collect()
    }

    pub fn has_conflicts(&self) -> bool {
        self.files.iter().any(|f| f.is_conflicted())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub sha: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
}

impl Author {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
        }
    }

    pub fn to_trailer(&self) -> String {
        format!(
            "Co-authored-by: {} <{}>",
            self.name.trim(),
            self.email.trim()
        )
    }

    pub fn parse_trailer(trailer_line: &str) -> Option<Self> {
        let trimmed = trailer_line.trim();
        let prefix = "Co-authored-by:";
        if !trimmed.starts_with(prefix) {
            return None;
        }
        let rest = trimmed[prefix.len()..].trim();
        let open_angle = rest.find('<')?;
        let close_angle = rest.rfind('>')?;
        if open_angle >= close_angle {
            return None;
        }

        let name = rest[..open_angle].trim();
        let email = rest[open_angle + 1..close_angle].trim();

        if name.is_empty() || email.is_empty() {
            return None;
        }

        Some(Self {
            name: name.to_string(),
            email: email.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub short_sha: String,
    pub summary: String,
    pub body: Option<String>,
    pub author: Author,
    pub date: String,
    pub co_authors: Vec<Author>,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitFileChange {
    pub path: String,
    pub status: String,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
    pub branch: String,
    pub sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub author: String,
    pub head_branch: String,
    pub base_branch: String,
    pub html_url: String,
    pub is_draft: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub author: String,
    pub html_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub url: String,
}
