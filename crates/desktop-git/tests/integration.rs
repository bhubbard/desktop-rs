use desktop_core::models::Author;
use desktop_git::GitClient;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn setup_repo() -> (tempfile::TempDir, GitClient) {
    let dir = tempdir().unwrap();
    let path = dir.path();

    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Integration Tester"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "tester@example.com"])
        .current_dir(path)
        .output()
        .unwrap();

    let client = GitClient::new(path.to_path_buf()).unwrap();
    (dir, client)
}

#[test]
fn test_workflow_staging_committing_branching() {
    let (dir, client) = setup_repo();

    // 1. Initial file
    let file1 = dir.path().join("file1.txt");
    fs::write(&file1, "First line\n").unwrap();

    let status = client.status().unwrap();
    assert_eq!(status.files.len(), 1);
    assert!(status.files[0].is_untracked());

    // 2. Stage file
    client.stage_file("file1.txt").unwrap();
    let status_after_stage = client.status().unwrap();
    assert!(status_after_stage.files[0].is_staged());

    // 3. Commit with co-author
    let co_authors = vec![Author::new("Octocat", "octocat@github.com")];
    let sha = client
        .commit(
            "feat: initial commit",
            Some("Commit body description"),
            &co_authors,
        )
        .unwrap();
    assert!(!sha.is_empty());

    let log = client.log(1).unwrap();
    assert_eq!(log[0].summary, "feat: initial commit");
    assert_eq!(log[0].co_authors.len(), 1);
    assert_eq!(log[0].co_authors[0].name, "Octocat");

    // 4. Create and switch to new branch
    client.create_branch("feature/test", None).unwrap();
    let status_branch = client.status().unwrap();
    assert_eq!(status_branch.branch, "feature/test");

    // 5. Modify file in new branch
    fs::write(&file1, "First line\nSecond line\n").unwrap();
    let diff = client.diff_unstaged(Some("file1.txt")).unwrap();
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].added_count(), 1);

    // 6. Stash changes
    client.stash_save(Some("WIP feature"), false).unwrap();
    let stashes = client.stash_list().unwrap();
    assert_eq!(stashes.len(), 1);
    assert!(stashes[0].message.contains("WIP feature"));

    // 7. Pop stash
    client.stash_pop(None).unwrap();
    let status_popped = client.status().unwrap();
    assert_eq!(status_popped.files.len(), 1);

    // 8. Commit and test undo
    client.stage_file("file1.txt").unwrap();
    client.commit("feat: added second line", None, &[]).unwrap();
    let log_before_undo = client.log(5).unwrap();
    assert_eq!(log_before_undo.len(), 2);

    client.undo_commit().unwrap();
    let log_after_undo = client.log(5).unwrap();
    assert_eq!(log_after_undo.len(), 1);

    // After undo, file should be staged
    let status_undone = client.status().unwrap();
    assert_eq!(status_undone.files.len(), 1);
    assert!(status_undone.files[0].is_staged());
}
