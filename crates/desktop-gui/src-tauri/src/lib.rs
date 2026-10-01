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
fn clone_repository(
    url: String,
    destination: String,
    state: State<'_, AppState>,
) -> Result<RepoSummary, String> {
    let dest_buf = PathBuf::from(&destination);
    let output = Command::new("git")
        .args(["clone", &url, &destination])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let git = GitClient::open_or_find(&dest_buf).map_err(|e| e.to_string())?;
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

    let repo_menu = Submenu::with_items(
        app,
        "Repository",
        true,
        &[
            &push,
            &pull,
            &fetch,
            &sep()?,
            &view_on_github,
            &open_terminal,
            &show_finder,
            &open_editor,
        ],
    )?;

    // 6. Branch Menu
    let new_branch = MenuItem::with_id(app, "create-branch", "New Branch…", true, Some("CmdOrControl+Shift+N"))?;
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

pub fn run() {
    let initial_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

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
            switch_repository,
            start_dragging,
            init_repository,
            clone_repository
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
