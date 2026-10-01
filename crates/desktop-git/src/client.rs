use crate::error::{GitError, Result};
use desktop_core::{
    commit::{extract_co_authors, format_commit_message},
    diff::parse_diff,
    models::{Author, Branch, Commit, FileChange, FileStatusType, RepositoryStatus, StashEntry},
    Diff, DiffHunk,
};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct GitClient {
    repo_path: PathBuf,
}

impl GitClient {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let repo_path = path.into();
        let client = Self { repo_path };
        if !client.is_git_repository() {
            return Err(GitError::NotAGitRepository(client.repo_path));
        }
        Ok(client)
    }

    pub fn open_or_find(start_path: impl AsRef<Path>) -> Result<Self> {
        let root = Self::find_repository_root(start_path.as_ref())?;
        Self::new(root)
    }

    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    pub fn repo_name(&self) -> String {
        self.repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository")
            .to_string()
    }

    pub fn is_git_repository(&self) -> bool {
        self.run_git(&["rev-parse", "--is-inside-work-tree"])
            .map(|out| out.trim() == "true")
            .unwrap_or(false)
    }

    pub fn find_repository_root(start: &Path) -> Result<PathBuf> {
        let mut curr = if start.is_file() {
            start.parent().unwrap_or(start).to_path_buf()
        } else {
            start.to_path_buf()
        };

        loop {
            if curr.join(".git").exists() {
                return Ok(curr);
            }
            if let Some(parent) = curr.parent() {
                curr = parent.to_path_buf();
            } else {
                break;
            }
        }

        Err(GitError::NotAGitRepository(start.to_path_buf()))
    }

    pub fn run_git(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(GitError::CommandFailed {
                command: format!("git {}", args.join(" ")),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    pub fn run_git_with_stdin(&self, args: &[&str], input: &str) -> Result<String> {
        let mut child = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(GitError::CommandFailed {
                command: format!("git {}", args.join(" ")),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    /// Fetches full repository status using git porcelain v2
    pub fn status(&self) -> Result<RepositoryStatus> {
        let output = self.run_git(&["status", "--porcelain=v2", "--branch", "--ahead-behind"])?;

        let mut branch = String::from("HEAD (detached)");
        let mut upstream = None;
        let mut ahead = 0;
        let mut behind = 0;
        let mut files = Vec::new();

        for line in output.lines() {
            if let Some(stripped) = line.strip_prefix("# branch.head ") {
                branch = stripped.trim().to_string();
            } else if let Some(stripped) = line.strip_prefix("# branch.upstream ") {
                upstream = Some(stripped.trim().to_string());
            } else if let Some(stripped) = line.strip_prefix("# branch.ab ") {
                let parts: Vec<&str> = stripped.split_whitespace().collect();
                for p in parts {
                    if let Some(val) = p.strip_prefix('+') {
                        ahead = val.parse().unwrap_or(0);
                    } else if let Some(val) = p.strip_prefix('-') {
                        behind = val.parse().unwrap_or(0);
                    }
                }
            } else if line.starts_with("1 ") {
                // Ordinary changed entry:
                // 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 {
                    let xy = parts[1];
                    let staged_char = xy.chars().next().unwrap_or('.');
                    let unstaged_char = xy.chars().nth(1).unwrap_or('.');
                    let path = parts[8..].join(" ");

                    files.push(FileChange::new(
                        path,
                        FileStatusType::from_porcelain_code(staged_char),
                        FileStatusType::from_porcelain_code(unstaged_char),
                    ));
                }
            } else if let Some(rest) = line.strip_prefix("2 ") {
                // Renamed or copied entry:
                // 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><tab><origPath>
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() >= 9 {
                    let xy = parts[0];
                    let staged_char = xy.chars().next().unwrap_or('.');
                    let unstaged_char = xy.chars().nth(1).unwrap_or('.');

                    let path_part = parts[8..].join(" ");
                    let (path, orig_path) = if let Some(tab_pos) = path_part.find('\t') {
                        (
                            path_part[..tab_pos].to_string(),
                            Some(path_part[tab_pos + 1..].to_string()),
                        )
                    } else {
                        (path_part, None)
                    };

                    let mut change = FileChange::new(
                        path,
                        FileStatusType::from_porcelain_code(staged_char),
                        FileStatusType::from_porcelain_code(unstaged_char),
                    );
                    if let Some(op) = orig_path {
                        change = change.with_old_path(op);
                    }
                    files.push(change);
                }
            } else if line.starts_with("u ") {
                // Unmerged entry
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 11 {
                    let path = parts[10..].join(" ");
                    files.push(FileChange::new(
                        path,
                        FileStatusType::Conflicted,
                        FileStatusType::Conflicted,
                    ));
                }
            } else if let Some(stripped) = line.strip_prefix("? ") {
                // Untracked entry
                let path = stripped.trim();
                files.push(FileChange::new(
                    path,
                    FileStatusType::Untracked,
                    FileStatusType::Untracked,
                ));
            }
        }

        Ok(RepositoryStatus {
            branch,
            upstream,
            ahead,
            behind,
            files,
        })
    }

    /// Generates diff for unstaged changes (working copy vs index)
    pub fn diff_unstaged(&self, path: Option<&str>) -> Result<Vec<Diff>> {
        let mut args = vec!["diff", "--no-color", "-p"];
        if let Some(p) = path {
            args.push("--");
            args.push(p);
        }
        let raw = self.run_git(&args)?;
        Ok(parse_diff(&raw))
    }

    /// Generates diff for staged changes (index vs HEAD)
    pub fn diff_staged(&self, path: Option<&str>) -> Result<Vec<Diff>> {
        let mut args = vec!["diff", "--cached", "--no-color", "-p"];
        if let Some(p) = path {
            args.push("--");
            args.push(p);
        }
        let raw = self.run_git(&args)?;
        Ok(parse_diff(&raw))
    }

    /// Generates diff for a specific commit
    pub fn diff_commit(&self, sha: &str) -> Result<Vec<Diff>> {
        let raw = self.run_git(&["show", "--no-color", "--format=", "--patch", sha])?;
        Ok(parse_diff(&raw))
    }

    pub fn stage_file(&self, path: &str) -> Result<()> {
        self.run_git(&["add", "--", path])?;
        Ok(())
    }

    pub fn stage_all(&self) -> Result<()> {
        self.run_git(&["add", "-A"])?;
        Ok(())
    }

    pub fn unstage_file(&self, path: &str) -> Result<()> {
        self.run_git(&["restore", "--staged", "--", path])
            .or_else(|_| self.run_git(&["reset", "HEAD", "--", path]))?;
        Ok(())
    }

    pub fn unstage_all(&self) -> Result<()> {
        self.run_git(&["restore", "--staged", "."])
            .or_else(|_| self.run_git(&["reset", "HEAD"]))?;
        Ok(())
    }

    pub fn discard_file(&self, path: &str) -> Result<()> {
        let status = self.status()?;
        let is_untracked = status
            .files
            .iter()
            .any(|f| f.path == path && f.is_untracked());

        if is_untracked {
            self.run_git(&["clean", "-f", "--", path])?;
        } else {
            self.unstage_file(path).ok();
            self.run_git(&["restore", "--", path])
                .or_else(|_| self.run_git(&["checkout", "--", path]))?;
        }
        Ok(())
    }

    /// Stages a single hunk by applying it directly to the git index
    pub fn stage_hunk(&self, hunk: &DiffHunk, file_path: &str) -> Result<()> {
        let patch = hunk.to_patch(file_path);
        self.run_git_with_stdin(&["apply", "--cached", "--unidiff-zero", "-"], &patch)?;
        Ok(())
    }

    /// Unstages a single hunk by reversing it from the index
    pub fn unstage_hunk(&self, hunk: &DiffHunk, file_path: &str) -> Result<()> {
        let patch = hunk.to_patch(file_path);
        self.run_git_with_stdin(
            &["apply", "--cached", "--reverse", "--unidiff-zero", "-"],
            &patch,
        )?;
        Ok(())
    }

    /// Discards a single hunk in the working tree
    pub fn discard_hunk(&self, hunk: &DiffHunk, file_path: &str) -> Result<()> {
        let patch = hunk.to_patch(file_path);
        self.run_git_with_stdin(&["apply", "--reverse", "--unidiff-zero", "-"], &patch)?;
        Ok(())
    }

    /// Creates a commit with summary, optional description, and co-authors
    pub fn commit(
        &self,
        summary: &str,
        description: Option<&str>,
        co_authors: &[Author],
    ) -> Result<String> {
        let message = format_commit_message(summary, description, co_authors);
        self.run_git_with_stdin(&["commit", "-F", "-"], &message)?;
        let sha = self.run_git(&["rev-parse", "HEAD"])?;
        Ok(sha.trim().to_string())
    }

    /// Undoes the last commit keeping all changes staged (soft reset)
    pub fn undo_commit(&self) -> Result<()> {
        self.run_git(&["reset", "--soft", "HEAD~1"])?;
        Ok(())
    }

    /// Reverts a specific commit by creating a new revert commit
    pub fn revert_commit(&self, sha: &str) -> Result<String> {
        self.run_git(&["revert", "--no-edit", sha])?;
        let new_sha = self.run_git(&["rev-parse", "HEAD"])?;
        Ok(new_sha.trim().to_string())
    }

    /// Lists branches
    pub fn branches(&self) -> Result<Vec<Branch>> {
        let format = "%(HEAD)|%(refname:short)|%(upstream:short)|%(objectname:short)";
        let output = self.run_git(&["branch", "-a", &format!("--format={format}")])?;

        let mut branches = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                let is_current = parts[0].trim() == "*";
                let name = parts[1].trim().to_string();
                let upstream = if parts[2].trim().is_empty() {
                    None
                } else {
                    Some(parts[2].trim().to_string())
                };
                let sha = if parts[3].trim().is_empty() {
                    None
                } else {
                    Some(parts[3].trim().to_string())
                };
                let is_remote = name.starts_with("origin/") || name.starts_with("upstream/");

                branches.push(Branch {
                    name,
                    is_current,
                    is_remote,
                    upstream,
                    sha,
                });
            }
        }
        Ok(branches)
    }

    pub fn current_branch(&self) -> Result<String> {
        let status = self.status()?;
        Ok(status.branch)
    }

    pub fn checkout(&self, branch_name: &str) -> Result<()> {
        self.run_git(&["checkout", branch_name])?;
        Ok(())
    }

    pub fn create_branch(&self, branch_name: &str, start_point: Option<&str>) -> Result<()> {
        let mut args = vec!["checkout", "-b", branch_name];
        if let Some(sp) = start_point {
            args.push(sp);
        }
        self.run_git(&args)?;
        Ok(())
    }

    pub fn delete_branch(&self, branch_name: &str, force: bool) -> Result<()> {
        let flag = if force { "-D" } else { "-d" };
        self.run_git(&["branch", flag, branch_name])?;
        Ok(())
    }

    pub fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<()> {
        self.run_git(&["branch", "-m", old_name, new_name])?;
        Ok(())
    }

    pub fn merge(&self, branch: &str) -> Result<String> {
        self.run_git(&["merge", "--no-ff", branch])
    }

    pub fn abort_merge(&self) -> Result<String> {
        self.run_git(&["merge", "--abort"])
    }

    pub fn create_tag(&self, name: &str, target_sha: Option<&str>) -> Result<()> {
        let mut args = vec!["tag", name];
        if let Some(sha) = target_sha {
            args.push(sha);
        }
        self.run_git(&args)?;
        Ok(())
    }

    pub fn delete_tag(&self, name: &str) -> Result<()> {
        self.run_git(&["tag", "-d", name])?;
        Ok(())
    }

    pub fn tags(&self) -> Result<Vec<String>> {
        let output = self.run_git(&["tag", "-l"])?;
        Ok(output
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect())
    }

    pub fn fetch(&self, remote: Option<&str>) -> Result<()> {
        let r = remote.unwrap_or("origin");
        self.run_git(&["fetch", r])?;
        Ok(())
    }

    pub fn pull(&self, remote: Option<&str>, branch: Option<&str>, rebase: bool) -> Result<String> {
        let mut args = vec!["pull"];
        if rebase {
            args.push("--rebase");
        }
        if let Some(r) = remote {
            args.push(r);
            if let Some(b) = branch {
                args.push(b);
            }
        }
        self.run_git(&args)
    }

    pub fn push(
        &self,
        remote: Option<&str>,
        branch: Option<&str>,
        set_upstream: bool,
    ) -> Result<String> {
        let mut args = vec!["push"];
        if set_upstream {
            args.push("-u");
        }
        if let Some(r) = remote {
            args.push(r);
            if let Some(b) = branch {
                args.push(b);
            }
        }
        self.run_git(&args)
    }

    pub fn remotes(&self) -> Result<Vec<desktop_core::models::Remote>> {
        let output = self.run_git(&["remote", "-v"])?;
        let mut map = std::collections::BTreeMap::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let url = parts[1].to_string();
                map.entry(name).or_insert(url);
            }
        }
        let remotes = map
            .into_iter()
            .map(|(name, url)| desktop_core::models::Remote { name, url })
            .collect();
        Ok(remotes)
    }

    pub fn stash_save(&self, message: Option<&str>, keep_index: bool) -> Result<()> {
        let mut args = vec!["stash", "push"];
        if keep_index {
            args.push("--keep-index");
        }
        if let Some(m) = message {
            args.push("-m");
            args.push(m);
        }
        self.run_git(&args)?;
        Ok(())
    }

    pub fn stash_pop(&self, index: Option<usize>) -> Result<()> {
        let mut args = vec!["stash", "pop"];
        let ref_name;
        if let Some(idx) = index {
            ref_name = format!("stash@{{{idx}}}");
            args.push(&ref_name);
        }
        self.run_git(&args)?;
        Ok(())
    }

    pub fn stash_drop(&self, index: usize) -> Result<()> {
        let ref_name = format!("stash@{{{idx}}}", idx = index);
        self.run_git(&["stash", "drop", &ref_name])?;
        Ok(())
    }

    pub fn stash_list(&self) -> Result<Vec<StashEntry>> {
        let output = self.run_git(&["stash", "list", "--format=%gd|%gs|%H"])?;
        let mut entries = Vec::new();

        for (i, line) in output.lines().enumerate() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                let message = parts[1].to_string();
                let sha = parts[2].to_string();

                let branch = if let Some(colon) = message.find(':') {
                    let prefix = &message[..colon];
                    if let Some(on_idx) = prefix.find("On ") {
                        prefix[on_idx + 3..].trim().to_string()
                    } else if let Some(wip_idx) = prefix.find("WIP on ") {
                        prefix[wip_idx + 7..].trim().to_string()
                    } else {
                        "HEAD".to_string()
                    }
                } else {
                    "HEAD".to_string()
                };

                entries.push(StashEntry {
                    index: i,
                    message,
                    branch,
                    sha,
                });
            }
        }
        Ok(entries)
    }

    /// Reads commit history
    pub fn log(&self, limit: usize) -> Result<Vec<Commit>> {
        let limit_str = limit.to_string();
        // Format: %H \x1F %h \x1F %an \x1F %ae \x1F %ad \x1F %s \x1F %b \x1F %P \x1E
        let output = self.run_git(&[
            "log",
            "-n",
            &limit_str,
            "--format=%H%x1f%h%x1f%an%x1f%ae%x1f%ad%x1f%s%x1f%b%x1f%P%x1e",
        ])?;

        let mut commits = Vec::new();
        for record in output.split('\x1e') {
            let record = record.trim();
            if record.is_empty() {
                continue;
            }

            let fields: Vec<&str> = record.split('\x1f').collect();
            if fields.len() >= 8 {
                let sha = fields[0].to_string();
                let short_sha = fields[1].to_string();
                let author_name = fields[2].to_string();
                let author_email = fields[3].to_string();
                let date = fields[4].to_string();
                let summary = fields[5].to_string();
                let raw_body = fields[6];
                let parents = fields[7]
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                let (body, co_authors) = extract_co_authors(raw_body);
                let opt_body = if body.trim().is_empty() {
                    None
                } else {
                    Some(body)
                };

                commits.push(Commit {
                    sha,
                    short_sha,
                    summary,
                    body: opt_body,
                    author: Author::new(author_name, author_email),
                    date,
                    co_authors,
                    parents,
                });
            }
        }

        Ok(commits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_repo() -> (tempfile::TempDir, GitClient) {
        let dir = tempdir().unwrap();
        let path = dir.path();

        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(path)
            .output()
            .unwrap();

        let client = GitClient::new(path.to_path_buf()).unwrap();
        (dir, client)
    }

    #[test]
    fn test_git_init_and_status() {
        let (_dir, client) = setup_test_repo();
        let status = client.status().unwrap();
        assert_eq!(status.branch, "main");
        assert!(status.is_clean());
    }

    #[test]
    fn test_commit_and_log() {
        let (dir, client) = setup_test_repo();
        let file_path = dir.path().join("hello.txt");
        std::fs::write(&file_path, "Hello world\n").unwrap();

        client.stage_file("hello.txt").unwrap();
        let status = client.status().unwrap();
        assert_eq!(status.files.len(), 1);
        assert!(status.files[0].is_staged());

        let sha = client
            .commit("Initial commit", Some("First commit description"), &[])
            .unwrap();
        assert!(!sha.is_empty());

        let log = client.log(5).unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].summary, "Initial commit");
        assert_eq!(log[0].author.name, "Test User");
    }

    #[test]
    fn test_commit_with_co_authors() {
        let (dir, client) = setup_test_repo();
        let file_path = dir.path().join("pair.txt");
        std::fs::write(&file_path, "Pair programming\n").unwrap();

        client.stage_file("pair.txt").unwrap();

        let co_authors = vec![Author::new("Partner", "partner@example.com")];
        client.commit("feat: pair work", None, &co_authors).unwrap();

        let log = client.log(1).unwrap();
        assert_eq!(log[0].co_authors.len(), 1);
        assert_eq!(log[0].co_authors[0].name, "Partner");
        assert_eq!(log[0].co_authors[0].email, "partner@example.com");
    }

    #[test]
    fn test_revert_commit() {
        let (dir, client) = setup_test_repo();
        let file_path = dir.path().join("feature.txt");
        std::fs::write(&file_path, "feature line\n").unwrap();
        client.stage_file("feature.txt").unwrap();
        let sha = client.commit("Add feature", None, &[]).unwrap();

        let revert_sha = client.revert_commit(&sha).unwrap();
        assert!(!revert_sha.is_empty());
        assert_ne!(revert_sha, sha);

        let log = client.log(2).unwrap();
        assert_eq!(log.len(), 2);
        assert!(log[0].summary.contains("Revert"));
        assert!(!file_path.exists());
    }

    #[test]
    fn test_branch_merge_and_rename() {
        let (dir, client) = setup_test_repo();
        let file_path = dir.path().join("base.txt");
        std::fs::write(&file_path, "base\n").unwrap();
        client.stage_file("base.txt").unwrap();
        client.commit("Base commit", None, &[]).unwrap();

        // Create feature branch
        client.create_branch("feature-1", None).unwrap();
        let feat_file = dir.path().join("feat.txt");
        std::fs::write(&feat_file, "feat content\n").unwrap();
        client.stage_file("feat.txt").unwrap();
        client.commit("Add feat", None, &[]).unwrap();

        // Rename branch
        client.rename_branch("feature-1", "feature-renamed").unwrap();
        let branches = client.branches().unwrap();
        assert!(branches.iter().any(|b| b.name == "feature-renamed"));

        // Switch back to main and merge
        client.checkout("main").unwrap();
        client.merge("feature-renamed").unwrap();

        assert!(feat_file.exists());
        let log = client.log(1).unwrap();
        assert!(log[0].summary.contains("Merge branch"));
    }

    #[test]
    fn test_tags() {
        let (dir, client) = setup_test_repo();
        let file_path = dir.path().join("base.txt");
        std::fs::write(&file_path, "base\n").unwrap();
        client.stage_file("base.txt").unwrap();
        client.commit("Initial", None, &[]).unwrap();

        client.create_tag("v1.0.0", None).unwrap();
        let tags = client.tags().unwrap();
        assert_eq!(tags, vec!["v1.0.0"]);

        client.delete_tag("v1.0.0").unwrap();
        assert!(client.tags().unwrap().is_empty());
    }
}

