use desktop_core::models::{Author, Branch, Commit, CommitFileChange, PullRequest, RepositoryStatus, StashEntry};
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
async fn commit(
    summary: String,
    description: Option<String>,
    co_authors: Vec<Author>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || {
        git.commit(&summary, description.as_deref(), &co_authors)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn undo_commit(state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || git.undo_commit().map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn revert_commit(sha: String, state: State<'_, AppState>) -> Result<String, String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || git.revert_commit(&sha).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
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
fn delete_branch(name: String, force: bool, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.delete_branch(&name, force).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_branch(old_name: String, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.rename_branch(&old_name, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn merge_branch(branch: String, state: State<'_, AppState>) -> Result<String, String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || git.merge(&branch).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn abort_merge(state: State<'_, AppState>) -> Result<String, String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || git.abort_merge().map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn stash_save(message: Option<String>, keep_index: bool, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || {
        git.stash_save(message.as_deref(), keep_index).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn stash_pop(index: Option<usize>, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || git.stash_pop(index).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn stash_drop(index: usize, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || git.stash_drop(index).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_stashes(state: State<'_, AppState>) -> Result<Vec<StashEntry>, String> {
    let git = get_git(&state)?;
    git.stash_list().map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_remote(state: State<'_, AppState>) -> Result<String, String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || {
        let status = git.status().map_err(|e| e.to_string())?;

        // 1. Ahead and not behind -> Push directly
        if status.ahead > 0 && status.behind == 0 {
            git.push(None, None, false).map_err(|e| e.to_string())?;
            return Ok(format!(
                "Pushed {} commit{}",
                status.ahead,
                if status.ahead == 1 { "" } else { "s" }
            ));
        }

        // 2. Behind and not ahead -> Pull directly
        if status.behind > 0 && status.ahead == 0 {
            git.pull(None, None, false).map_err(|e| e.to_string())?;
            return Ok(format!(
                "Pulled {} commit{}",
                status.behind,
                if status.behind == 1 { "" } else { "s" }
            ));
        }

        // 3. Neither ahead nor behind -> Fetch from origin
        if status.ahead == 0 && status.behind == 0 {
            git.fetch(None).map_err(|e| e.to_string())?;
            let status_after = git.status().map_err(|e| e.to_string())?;
            if status_after.behind > 0 {
                return Ok(format!(
                    "Fetched from origin ({} commit{} to pull)",
                    status_after.behind,
                    if status_after.behind == 1 { "" } else { "s" }
                ));
            } else if status_after.ahead > 0 {
                return Ok(format!(
                    "Fetched from origin ({} commit{} to push)",
                    status_after.ahead,
                    if status_after.ahead == 1 { "" } else { "s" }
                ));
            } else {
                return Ok(String::from("Fetched from origin (up to date)"));
            }
        }

        // 4. Diverged (both ahead and behind) -> Fetch, Pull, Push
        git.fetch(None).map_err(|e| e.to_string())?;
        git.pull(None, None, false).map_err(|e| e.to_string())?;
        git.push(None, None, false).map_err(|e| e.to_string())?;
        Ok(String::from("Synchronized with origin (pulled & pushed)"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn push(state: State<'_, AppState>) -> Result<String, String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || {
        git.push(None, None, false).map_err(|e| e.to_string())?;
        Ok(String::from("Pushed commits to origin"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn pull(state: State<'_, AppState>) -> Result<String, String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || {
        git.pull(None, None, false).map_err(|e| e.to_string())?;
        Ok(String::from("Pulled commits from origin"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn fetch(state: State<'_, AppState>) -> Result<String, String> {
    let git = get_git(&state)?;
    tokio::task::spawn_blocking(move || {
        git.fetch(None).map_err(|e| e.to_string())?;
        Ok(String::from("Fetched from origin"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_commits(limit: Option<usize>, state: State<'_, AppState>) -> Result<Vec<Commit>, String> {
    let git = get_git(&state)?;
    git.log(limit.unwrap_or(50)).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_commit_files(sha: String, state: State<'_, AppState>) -> Result<Vec<CommitFileChange>, String> {
    let git = get_git(&state)?;
    git.commit_files(&sha).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_commit_diff(
    sha: String,
    file_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Diff>, String> {
    let git = get_git(&state)?;
    git.diff_commit(&sha, file_path.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn apply_patch(
    patch: String,
    cached: bool,
    reverse: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let git = get_git(&state)?;
    git.apply_patch(&patch, cached, reverse)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ignore_file(pattern: String, state: State<'_, AppState>) -> Result<(), String> {
    let git = get_git(&state)?;
    git.ignore_file(&pattern).map_err(|e| e.to_string())
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

fn get_config_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("desktop-rs")
    } else {
        PathBuf::from(".desktop-rs")
    }
}

fn get_saved_repo_paths() -> Vec<PathBuf> {
    let file = get_config_dir().join("repositories.json");
    if let Ok(data) = std::fs::read_to_string(&file) {
        if let Ok(paths) = serde_json::from_str::<Vec<PathBuf>>(&data) {
            return paths.into_iter().filter(|p| p.join(".git").exists()).collect();
        }
    }
    Vec::new()
}

fn save_repo_paths(paths: &[PathBuf]) {
    let dir = get_config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join("repositories.json");
    let _ = std::fs::write(file, serde_json::to_string_pretty(paths).unwrap_or_default());
}

fn add_saved_repo(path: &PathBuf) {
    let mut paths = get_saved_repo_paths();
    if !paths.contains(path) && path.join(".git").exists() {
        paths.push(path.clone());
        save_repo_paths(&paths);
    }
}

#[tauri::command]
fn switch_repository(new_path: String, state: State<'_, AppState>) -> Result<RepoSummary, String> {
    let path_buf = PathBuf::from(&new_path);
    let git = GitClient::open_or_find(&path_buf).map_err(|e| e.to_string())?;
    let status = git.status().map_err(|e| e.to_string())?;

    let target_path = git.repo_path().to_path_buf();
    add_saved_repo(&target_path);

    if let Ok(mut curr) = state.current_repo.lock() {
        *curr = target_path;
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

#[derive(Serialize, Deserialize, Clone)]
pub struct RepoItem {
    pub name: String,
    pub path: String,
    pub is_current: bool,
}

#[tauri::command]
fn get_repositories(state: State<'_, AppState>) -> Result<Vec<RepoItem>, String> {
    let current_path = state.current_repo.lock().map_err(|e| e.to_string())?.clone();
    add_saved_repo(&current_path);

    let mut saved = get_saved_repo_paths();

    if let Some(parent) = current_path.parent() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.join(".git").exists() && !saved.contains(&p) {
                    saved.push(p);
                }
            }
        }
    }

    save_repo_paths(&saved);

    let mut repos = Vec::new();
    for p in saved {
        let is_current = p == current_path;
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("repo").to_string();
        repos.push(RepoItem {
            name,
            path: p.display().to_string(),
            is_current,
        });
    }

    repos.sort_by(|a, b| {
        if a.is_current {
            std::cmp::Ordering::Less
        } else if b.is_current {
            std::cmp::Ordering::Greater
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(repos)
}

#[tauri::command]
fn remove_repository(path: String, state: State<'_, AppState>) -> Result<Vec<RepoItem>, String> {
    let target = PathBuf::from(&path);
    let mut saved = get_saved_repo_paths();
    saved.retain(|p| p != &target);
    save_repo_paths(&saved);
    get_repositories(state)
}

#[tauri::command]
fn start_dragging(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::Emitter;

#[tauri::command]
fn init_repository(
    name: String,
    parent_path: String,
    init_readme: bool,
    state: State<'_, AppState>,
) -> Result<RepoSummary, String> {
    let target_dir = PathBuf::from(&parent_path).join(&name);
    std::fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let output = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(&target_dir)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    if init_readme {
        let readme_path = target_dir.join("README.md");
        std::fs::write(&readme_path, format!("# {name}\n")).map_err(|e| e.to_string())?;
        let _ = Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&target_dir)
            .output();
        let _ = Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&target_dir)
            .output();
    }

    let git = GitClient::new(&target_dir).map_err(|e| e.to_string())?;
    let status = git.status().map_err(|e| e.to_string())?;

    if let Ok(mut curr) = state.current_repo.lock() {
        *curr = target_dir;
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

#[tauri::command]
async fn clone_repository(
    url: String,
    destination: String,
    state: State<'_, AppState>,
) -> Result<RepoSummary, String> {
    let dest_buf = PathBuf::from(&destination);
    let dest_clone = dest_buf.clone();

    let (repo_name, repo_path, branch, upstream, ahead, behind, is_clean, total_changes) =
        tokio::task::spawn_blocking(move || -> Result<_, String> {
            let output = Command::new("git")
                .args(["clone", &url, &destination])
                .output()
                .map_err(|e| e.to_string())?;

            if !output.status.success() {
                return Err(String::from_utf8_lossy(&output.stderr).to_string());
            }

            let git = GitClient::open_or_find(&dest_buf).map_err(|e| e.to_string())?;
            let status = git.status().map_err(|e| e.to_string())?;
            let is_clean = status.is_clean();
            let total_changes = status.total_changes();

            Ok((
                git.repo_name(),
                git.repo_path().display().to_string(),
                status.branch,
                status.upstream,
                status.ahead,
                status.behind,
                is_clean,
                total_changes,
            ))
        })
        .await
        .map_err(|e| e.to_string())??;

    if let Ok(mut curr) = state.current_repo.lock() {
        *curr = dest_clone;
    }

    Ok(RepoSummary {
        name: repo_name,
        path: repo_path,
        branch,
        upstream,
        ahead,
        behind,
        is_clean,
        total_changes,
    })
}

fn build_app_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let sep = || PredefinedMenuItem::separator(app);

    // 1. App Menu (GitHub Desktop)
    let about = PredefinedMenuItem::about(app, Some("GitHub Desktop"), None)?;
    let settings = MenuItem::with_id(app, "preferences", "Settings…", true, Some("CmdOrControl+,"))?;
    let services = PredefinedMenuItem::services(app, None)?;
    let hide = PredefinedMenuItem::hide(app, Some("Hide GitHub Desktop"))?;
    let hide_others = PredefinedMenuItem::hide_others(app, Some("Hide Others"))?;
    let show_all = PredefinedMenuItem::show_all(app, Some("Show All"))?;
    let quit = PredefinedMenuItem::quit(app, Some("Quit GitHub Desktop"))?;

    let app_menu = Submenu::with_items(
        app,
        "GitHub Desktop",
        true,
        &[
            &about,
            &sep()?,
            &settings,
            &sep()?,
            &services,
            &sep()?,
            &hide,
            &hide_others,
            &show_all,
            &sep()?,
            &quit,
        ],
    )?;

    // 2. File Menu
    let new_repo = MenuItem::with_id(app, "new-repository", "New Repository…", true, Some("CmdOrControl+N"))?;
    let add_local_repo = MenuItem::with_id(app, "add-local-repository", "Add Local Repository…", true, Some("CmdOrControl+O"))?;
    let clone_repo = MenuItem::with_id(app, "clone-repository", "Clone Repository…", true, Some("CmdOrControl+Shift+O"))?;
    let close_window = PredefinedMenuItem::close_window(app, None)?;

    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &new_repo,
            &sep()?,
            &add_local_repo,
            &clone_repo,
            &sep()?,
            &close_window,
        ],
    )?;

    // 3. Edit Menu
    let undo = PredefinedMenuItem::undo(app, None)?;
    let redo = PredefinedMenuItem::redo(app, None)?;
    let cut = PredefinedMenuItem::cut(app, None)?;
    let copy = PredefinedMenuItem::copy(app, None)?;
    let paste = PredefinedMenuItem::paste(app, None)?;
    let select_all = PredefinedMenuItem::select_all(app, None)?;

    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &undo,
            &redo,
            &sep()?,
            &cut,
            &copy,
            &paste,
            &select_all,
        ],
    )?;

    // 4. View Menu
    let show_changes = MenuItem::with_id(app, "show-changes", "Changes", true, Some("CmdOrControl+1"))?;
    let show_history = MenuItem::with_id(app, "show-history", "History", true, Some("CmdOrControl+2"))?;
    let show_repos = MenuItem::with_id(app, "show-repository-list", "Repository List", true, Some("CmdOrControl+T"))?;
    let show_branches = MenuItem::with_id(app, "show-branches-list", "Branches List", true, Some("CmdOrControl+B"))?;
    let toggle_fullscreen = PredefinedMenuItem::fullscreen(app, None)?;
    let reload = MenuItem::with_id(app, "reload-window", "Reload", true, Some("CmdOrControl+Alt+R"))?;

    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &show_changes,
            &show_history,
            &show_repos,
            &show_branches,
            &sep()?,
            &toggle_fullscreen,
            &sep()?,
            &reload,
        ],
    )?;

    // 5. Repository Menu
    let push = MenuItem::with_id(app, "push", "Push", true, Some("CmdOrControl+P"))?;
    let pull = MenuItem::with_id(app, "pull", "Pull", true, Some("CmdOrControl+Shift+P"))?;
    let fetch = MenuItem::with_id(app, "fetch", "Fetch", true, Some("CmdOrControl+Shift+T"))?;
    let view_on_github = MenuItem::with_id(app, "view-repository-on-github", "View on GitHub", true, Some("CmdOrControl+Shift+G"))?;
    let open_terminal = MenuItem::with_id(app, "open-in-shell", "Open in Terminal", true, Some("Control+`"))?;
    let show_finder = MenuItem::with_id(app, "open-working-directory", "Show in Finder", true, Some("CmdOrControl+Shift+F"))?;
    let open_editor = MenuItem::with_id(app, "open-external-editor", "Open in External Editor", true, Some("CmdOrControl+Shift+A"))?;
    let revert_commit_item = MenuItem::with_id(app, "revert-commit", "Revert Selected Commit…", true, Some("CmdOrControl+Shift+R"))?;

    let repo_menu = Submenu::with_items(
        app,
        "Repository",
        true,
        &[
            &push,
            &pull,
            &fetch,
            &sep()?,
            &revert_commit_item,
            &sep()?,
            &view_on_github,
            &open_terminal,
            &show_finder,
            &open_editor,
        ],
    )?;

    // 6. Branch Menu
    let new_branch = MenuItem::with_id(app, "create-branch", "New Branch…", true, Some("CmdOrControl+Shift+N"))?;
    let merge_branch_item = MenuItem::with_id(app, "merge-into-current-branch", "Merge into Current Branch…", true, Some("CmdOrControl+Shift+M"))?;
    let discard_all = MenuItem::with_id(app, "discard-all-changes", "Discard All Changes…", true, Some("CmdOrControl+Shift+Backspace"))?;
    let stash_all = MenuItem::with_id(app, "stash-all-changes", "Stash All Changes…", true, Some("CmdOrControl+Shift+S"))?;
    let compare_github = MenuItem::with_id(app, "compare-on-github", "Compare on GitHub", true, Some("CmdOrControl+Shift+C"))?;
    let create_pr = MenuItem::with_id(app, "create-pull-request", "Create Pull Request", true, Some("CmdOrControl+R"))?;

    let branch_menu = Submenu::with_items(
        app,
        "Branch",
        true,
        &[
            &new_branch,
            &merge_branch_item,
            &sep()?,
            &discard_all,
            &stash_all,
            &sep()?,
            &compare_github,
            &create_pr,
        ],
    )?;

    // 7. Window Menu
    let minimize = PredefinedMenuItem::minimize(app, None)?;
    let bring_all = PredefinedMenuItem::bring_all_to_front(app, None)?;

    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &minimize,
            &sep()?,
            &bring_all,
        ],
    )?;

    // 8. Help Menu
    let report_issue = MenuItem::with_id(app, "report-issue", "Report Issue…", true, None::<&str>)?;
    let docs = MenuItem::with_id(app, "show-docs", "GitHub Desktop Documentation", true, None::<&str>)?;

    let help_menu = Submenu::with_items(
        app,
        "Help",
        true,
        &[
            &report_issue,
            &docs,
        ],
    )?;

    Menu::with_items(
        app,
        &[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &repo_menu,
            &branch_menu,
            &window_menu,
            &help_menu,
        ],
    )
}

fn find_initial_repo_path() -> PathBuf {
    if let Ok(dir) = std::env::current_dir() {
        if GitClient::open_or_find(&dir).is_ok() {
            return dir;
        }
    }
    let default_project = PathBuf::from("/Users/bhubbard/PROJECTS/desktop-rs");
    if default_project.exists() && GitClient::open_or_find(&default_project).is_ok() {
        return default_project;
    }
    PathBuf::from(".")
}

pub fn run() {
    let initial_path = find_initial_repo_path();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .menu(build_app_menu)
        .on_menu_event(|app, event| {
            let _ = app.emit("menu-event", event.id().as_ref());
        })
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
            revert_commit,
            get_branches,
            checkout_branch,
            create_branch,
            delete_branch,
            rename_branch,
            merge_branch,
            abort_merge,
            stash_save,
            stash_pop,
            stash_drop,
            get_stashes,
            sync_remote,
            push,
            pull,
            fetch,
            get_commits,
            get_commit_files,
            get_commit_diff,
            apply_patch,
            ignore_file,
            get_github_prs,
            checkout_pr,
            open_in_editor,
            reveal_in_finder,
            open_in_terminal,
            switch_repository,
            get_repositories,
            remove_repository,
            start_dragging,
            init_repository,
            clone_repository
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
