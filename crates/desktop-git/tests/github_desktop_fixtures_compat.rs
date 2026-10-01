use desktop_core::models::FileStatusType;
use desktop_git::GitClient;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// Helper replicating GitHub Desktop's `setupFixtureRepository`:
/// 1. Creates a temporary directory.
/// 2. Recursively copies the named fixture from `tests/fixtures/<name>`.
/// 3. Renames all `_git` folders to `.git` so git recognizes the repository.
fn setup_fixture(fixture_name: &str) -> (tempfile::TempDir, GitClient) {
    let dir = tempdir().expect("failed to create temp dir");
    let target = dir.path();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("parent")
        .parent()
        .expect("workspace root");
    let src_fixture = workspace_root.join("tests").join("fixtures").join(fixture_name);

    assert!(
        src_fixture.exists(),
        "Fixture path does not exist: {:?}",
        src_fixture
    );

    copy_dir_recursive(&src_fixture, target).expect("failed to copy fixture");

    // Rename _git to .git if present
    let git_hidden = target.join("_git");
    if git_hidden.exists() {
        fs::rename(&git_hidden, target.join(".git")).expect("failed to rename _git to .git");
    }

    let client = GitClient::new(target.to_path_buf()).expect("failed to init GitClient");
    (dir, client)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Tests working directory status against GitHub Desktop's `repo-with-changes` fixture
/// (matches GitHub Desktop `git/status-test.ts`)
#[test]
fn test_fixture_repo_with_changes_status() {
    let (_dir, client) = setup_fixture("repo-with-changes");
    let status = client.status().expect("status should succeed");

    assert_eq!(status.branch, "master");
    assert!(!status.is_clean());

    // Verify expected modified, deleted, staged, and untracked files
    let paths: Vec<&str> = status.files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"deleted-file.md"), "missing deleted-file.md");
    assert!(paths.contains(&"modified-file.md"), "missing modified-file.md");
    assert!(paths.contains(&"staged-file.md"), "missing staged-file.md");
    assert!(paths.contains(&"new-file.md"), "missing new-file.md");

    let deleted = status.files.iter().find(|f| f.path == "deleted-file.md").unwrap();
    assert_eq!(deleted.unstaged_status, FileStatusType::Deleted);

    let modified = status.files.iter().find(|f| f.path == "modified-file.md").unwrap();
    assert_eq!(modified.unstaged_status, FileStatusType::Modified);

    let staged = status.files.iter().find(|f| f.path == "staged-file.md").unwrap();
    assert!(staged.is_staged());

    let untracked = status.files.iter().find(|f| f.path == "new-file.md").unwrap();
    assert!(untracked.is_untracked());
}

/// Tests git diff generation against GitHub Desktop's `repo-with-changes` fixture
/// (matches GitHub Desktop `git/diff-test.ts`)
#[test]
fn test_fixture_repo_with_changes_diff() {
    let (_dir, client) = setup_fixture("repo-with-changes");
    let diffs = client
        .diff_unstaged(Some("modified-file.md"))
        .expect("diff should succeed");

    assert_eq!(diffs.len(), 1);
    let diff = &diffs[0];
    assert!(!diff.hunks.is_empty(), "expected hunks in modified file");
    assert!(diff.added_count() > 0 || diff.deleted_count() > 0);
}

/// Tests commit history and branch detection against GitHub Desktop's `test-repo` fixture
/// (matches GitHub Desktop `git/log-test.ts`)
#[test]
fn test_fixture_test_repo_log_and_branches() {
    let (_dir, client) = setup_fixture("test-repo");
    let status = client.status().expect("status should succeed");

    assert_eq!(status.branch, "master");
    assert!(status.is_clean());

    let commits = client.log(10).expect("log should succeed");
    assert_eq!(commits.len(), 5, "test-repo should have exactly 5 commits");

    assert_eq!(commits[0].summary, "No content");
    assert!(commits[0].sha.starts_with("04c7629"));

    assert_eq!(commits[1].summary, "Remove attributes");
    assert!(commits[1].sha.starts_with("203168a"));

    assert_eq!(commits[2].summary, "Binary?");
    assert!(commits[2].sha.starts_with("7036e98"));

    assert_eq!(commits[3].summary, "Attributes");
    assert!(commits[3].sha.starts_with("0ca9c57"));

    assert_eq!(commits[4].summary, "first");
    assert!(commits[4].sha.starts_with("7cd6640"));
}

/// Tests detached HEAD parsing against GitHub Desktop's `detached-head` fixture
/// (matches GitHub Desktop detached-head tests)
#[test]
fn test_fixture_detached_head() {
    let (_dir, client) = setup_fixture("detached-head");
    let status = client.status().expect("status should succeed");

    assert!(
        status.branch.contains("detached"),
        "branch should indicate detached: {}",
        status.branch
    );

    let commits = client.log(1).expect("log should succeed");
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].sha, "2acb028231d408aaa865f9538b1c89de5a2b9da8");
}

/// Tests remote discovery against GitHub Desktop's `repo-with-multiple-remotes` fixture
/// (matches GitHub Desktop `git/remote-test.ts`)
#[test]
fn test_fixture_repo_with_multiple_remotes() {
    let (_dir, client) = setup_fixture("repo-with-multiple-remotes");
    let remotes = client.remotes().expect("remotes should succeed");

    assert_eq!(remotes.len(), 2, "expected 2 remotes (bassoon and origin)");
    let names: Vec<&str> = remotes.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, vec!["bassoon", "origin"]);

    for r in &remotes {
        assert!(
            r.url.ends_with("shiftkey/friendly-bassoon.git"),
            "unexpected remote url: {}",
            r.url
        );
    }
}

/// Tests branch listing against GitHub Desktop's `branch-prune-tests` fixture
/// (matches GitHub Desktop `git/branch-test.ts`)
#[test]
fn test_fixture_branch_prune_tests() {
    let (_dir, client) = setup_fixture("branch-prune-tests");
    let branches = client.branches().expect("branches should succeed");

    let branch_names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();
    assert!(branch_names.contains(&"master"));
    assert!(branch_names.contains(&"dev"));
    assert!(branch_names.contains(&"devel"));
    assert!(branch_names.contains(&"develop"));
    assert!(branch_names.contains(&"release"));
}

/// Tests staging, committing, and log progression on `repo-with-changes`
#[test]
fn test_fixture_staging_and_committing() {
    let (_dir, client) = setup_fixture("repo-with-changes");

    // Stage new-file.md
    client.stage_file("new-file.md").expect("stage new-file");
    let status = client.status().expect("status");
    let new_file = status.files.iter().find(|f| f.path == "new-file.md").unwrap();
    assert!(new_file.is_staged());

    // Commit
    let sha = client
        .commit("feat: add new file", Some("description"), &[])
        .expect("commit");
    assert!(!sha.is_empty());

    let commits = client.log(1).expect("log");
    assert_eq!(commits[0].sha, sha);
    assert_eq!(commits[0].summary, "feat: add new file");
}

/// Tests stash lifecycle on `repo-with-changes`
/// (matches GitHub Desktop `git/stash-test.ts`)
#[test]
fn test_fixture_stash_lifecycle() {
    let (_dir, client) = setup_fixture("repo-with-changes");

    // Save stash
    client.stash_save(Some("my-stash-work"), false).expect("stash save");

    let stashes = client.stash_list().expect("stash list");
    assert_eq!(stashes.len(), 1);
    assert!(stashes[0].message.contains("my-stash-work"));

    // Pop stash
    client.stash_pop(None).expect("stash pop");
    let status_after = client.status().expect("status after pop");
    assert!(!status_after.is_clean());
}
