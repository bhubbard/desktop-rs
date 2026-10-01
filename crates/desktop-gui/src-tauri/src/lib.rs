use desktop_core::models::{Author, Branch, Commit, PullRequest, RepositoryStatus};
use desktop_core::Diff;
use desktop_git::GitClient;
use desktop_github::GitHubClient;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub current_repo: Mutex<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct RepoSummary {
    pub name: String,
    pub path: String,
    pub branch: String,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub is_clean: bool,
    pub total_changes: usize,
}

fn get_git(state: &State<'_, AppState>) -> Result<GitClient, String> {
    let path = state.current_repo.lock().map_err(|e| e.to_string())?.clone();
    GitClient::open_or_find(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_repo_summary(state: State<'_, AppState>) -> Result<RepoSummary, String> {
    let git = get_git(&state)?;
    let status = git.status().map_err(|e| e.to_string())?;
    let is_clean = status.is_clean();
    let total_changes = status.total_changes();
    Ok(RepoSummary {
        name: git.repo_name(),
        path: git.repo_path().display().to_string(),
        branch: status.branch,
        upstream: status.upstream,
        ahead: status.ahead,
        behind: status.behind,
        is_clean,
        total_changes,
    })
}

#[tauri::command]
fn get_status(state: State<'_, AppState>) -> Result<RepositoryStatus, String> {
    let git = get_git(&state)?;
    git.status().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_diff(
    file_path: Option<String>,
    staged: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Diff>, String> {
    let git = get_git(&state)?;
    let p = file_path.as_deref();
    if staged {
        git.diff_staged(p).map_err(|e| e.to_string())
    } else {
        let diffs = git.diff_unstaged(p).map_err(|e| e.to_string())?;
        if diffs.is_empty() {
            // Fallback to staged if unstaged is empty
            git.diff_staged(p).map_err(|e| e.to_string())
        } else {
            Ok(diffs)
        }
    }
}

#[tauri::command]
fn stage_file(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.stage_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn unstage_file(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.unstage_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn stage_all(state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.stage_all().map_err(|e| e.to_string())
}

#[tauri::command]
fn unstage_all(state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.unstage_all().map_err(|e| e.to_string())
}

#[tauri::command]
fn discard_file(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.discard_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn commit(
    summary: String,
    description: Option<String>,
    co_authors: Vec<Author>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let git = get_git(&state)?;
    git.commit(&summary, description.as_deref(), &co_authors)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn undo_commit(state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.undo_commit().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_branches(state: State<'_, AppState>) -> Result<Vec<Branch>, String> {
    let git = get_git(&state)?;
    git.branches().map_err(|e| e.to_string())
}

#[tauri::command]
fn checkout_branch(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.checkout(&name).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_branch(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.create_branch(&name, None).map_err(|e| e.to_string())
}

#[tauri::command]
fn sync_remote(state: State<'_, AppState>) -> Result<String, String> {
    let git = get_git(&state)?;
    git.fetch(None).map_err(|e| e.to_string())?;

    let status = git.status().map_err(|e| e.to_string())?;
    let mut message = String::from("Fetched from origin");

    if status.behind > 0 {
        git.pull(None, None, false).map_err(|e| e.to_string())?;
        message = format!("Pulled {} commits", status.behind);
    }
    if status.ahead > 0 {
        git.push(None, None, false).map_err(|e| e.to_string())?;
        message = format!("Pushed {} commits", status.ahead);
    }

    Ok(message)
}

#[tauri::command]
fn get_commits(limit: Option<usize>, state: State<'_, AppState>) -> Result<Vec<Commit>, String> {
    let git = get_git(&state)?;
    git.log(limit.unwrap_or(50)).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_commit_diff(sha: String, state: State<'_, AppState>) -> Result<Vec<Diff>, String> {
    let git = get_git(&state)?;
    git.diff_commit(&sha).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_github_prs(state: State<'_, AppState>) -> Result<Vec<PullRequest>, String> {
    let git = get_git(&state)?;
    let remotes = git.run_git(&["remote", "get-url", "origin"]).ok();
    let url = remotes
        .as_deref()
        .ok_or_else(|| "No origin remote configured".to_string())?;
    let (owner, repo) = GitHubClient::parse_owner_repo(url)
        .ok_or_else(|| "Could not parse GitHub repo".to_string())?;

    let client = GitHubClient::from_env();
    client
        .list_pull_requests(&owner, &repo, Some("open"))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn checkout_pr(number: u64, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    let pr_ref = format!("refs/pull/{number}/head:pr/{number}");
    git.run_git(&["fetch", "origin", &pr_ref])
        .map_err(|e| e.to_string())?;
    let branch = format!("pr/{number}");
    git.checkout(&branch).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn open_in_editor(path: Option<String>, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    let target = if let Some(p) = path {
        git.repo_path().join(p)
    } else {
        git.repo_path().to_path_buf()
    };

    // Try Zed, then VS Code, then macOS default open
    let _ = Command::new("zed")
        .arg(&target)
        .status()
        .or_else(|_| Command::new("code").arg(&target).status())
        .or_else(|_| Command::new("open").arg(&target).status());

    Ok(())
}

#[tauri::command]
fn reveal_in_finder(path: Option<String>, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    let target = if let Some(p) = path {
        git.repo_path().join(p)
    } else {
        git.repo_path().to_path_buf()
    };

    let _ = Command::new("open").arg("-R").arg(&target).status();
    Ok(())
}

#[tauri::command]
fn open_in_terminal(state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    let path = git.repo_path();
    let _ = Command::new("open")
        .arg("-a")
        .arg("Terminal")
        .arg(path)
        .status();
    Ok(())
}

#[tauri::command]
fn switch_repository(new_path: String, state: State<'_, AppState>) -> Result<RepoSummary, String> {
    let path_buf = PathBuf::from(&new_path);
    let git = GitClient::open_or_find(&path_buf).map_err(|e| e.to_string())?;
    let status = git.status().map_err(|e| e.to_string())?;

    if let Ok(mut curr) = state.current_repo.lock() {
        *curr = git.repo_path().to_path_buf();
    }

    let is_clean = status.is_clean();
    let total_changes = status.total_changes();
    Ok(RepoSummary {
        name: git.repo_name(),
        path: git.repo_path().display().to_string(),
        branch: status.branch,
        upstream: status.upstream,
        ahead: status.ahead,
        behind: status.behind,
        is_clean,
        total_changes,
    })
}

pub fn run() {
    let initial_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            current_repo: Mutex::new(initial_path),
        })
        .invoke_handler(tauri::generate_handler![
            get_repo_summary,
            get_status,
            get_diff,
            stage_file,
            unstage_file,
            stage_all,
            unstage_all,
            discard_file,
            commit,
            undo_commit,
            get_branches,
            checkout_branch,
            create_branch,
            sync_remote,
            get_commits,
            get_commit_diff,
            get_github_prs,
            checkout_pr,
            open_in_editor,
            reveal_in_finder,
            open_in_terminal,
            switch_repository
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
