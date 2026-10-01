use desktop_core::{
    models::{Author, Branch, Commit, FileChange, RepositoryStatus},
    Diff,
};
use desktop_git::GitClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Changes,
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    FileList,
    DiffViewer,
    CommitSummary,
    CommitDescription,
    BranchModal,
}

pub struct App {
    pub git: GitClient,
    pub status: RepositoryStatus,
    pub active_tab: ActiveTab,
    pub focused_pane: FocusedPane,

    // Changes tab state
    pub selected_file_index: usize,
    pub current_diffs: Vec<Diff>,
    pub diff_scroll: u16,

    // Commit box state
    pub commit_summary: String,
    pub commit_description: String,
    pub commit_co_authors: Vec<Author>,

    // History tab state
    pub commits: Vec<Commit>,
    pub selected_commit_index: usize,

    // Branch switcher modal
    pub show_branch_modal: bool,
    pub branches: Vec<Branch>,
    pub selected_branch_index: usize,
    pub branch_filter: String,

    // Status messages / feedback toast
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub should_quit: bool,
}

impl App {
    pub fn new(git: GitClient) -> anyhow::Result<Self> {
        let status = git.status().unwrap_or(RepositoryStatus {
            branch: "unknown".to_string(),
            upstream: None,
            ahead: 0,
            behind: 0,
            files: Vec::new(),
        });

        let mut app = Self {
            git,
            status,
            active_tab: ActiveTab::Changes,
            focused_pane: FocusedPane::FileList,
            selected_file_index: 0,
            current_diffs: Vec::new(),
            diff_scroll: 0,
            commit_summary: String::new(),
            commit_description: String::new(),
            commit_co_authors: Vec::new(),
            commits: Vec::new(),
            selected_commit_index: 0,
            show_branch_modal: false,
            branches: Vec::new(),
            selected_branch_index: 0,
            branch_filter: String::new(),
            status_message: None,
            should_quit: false,
        };

        app.refresh_status()?;
        app.load_commits(50).ok();
        Ok(app)
    }

    pub fn refresh_status(&mut self) -> anyhow::Result<()> {
        self.status = self.git.status()?;
        if self.selected_file_index >= self.status.files.len() && !self.status.files.is_empty() {
            self.selected_file_index = self.status.files.len() - 1;
        }
        self.load_selected_file_diff().ok();
        Ok(())
    }

    pub fn load_commits(&mut self, limit: usize) -> anyhow::Result<()> {
        self.commits = self.git.log(limit)?;
        if self.selected_commit_index >= self.commits.len() && !self.commits.is_empty() {
            self.selected_commit_index = self.commits.len() - 1;
        }
        Ok(())
    }

    pub fn load_selected_file_diff(&mut self) -> anyhow::Result<()> {
        self.diff_scroll = 0;
        match self.active_tab {
            ActiveTab::Changes => {
                if let Some(file) = self.status.files.get(self.selected_file_index) {
                    let path = &file.path;
                    let mut diffs = if file.is_staged() {
                        self.git.diff_staged(Some(path)).unwrap_or_default()
                    } else {
                        self.git.diff_unstaged(Some(path)).unwrap_or_default()
                    };

                    if diffs.is_empty() {
                        // Try either staged or unstaged
                        diffs = self.git.diff_unstaged(Some(path)).unwrap_or_default();
                    }
                    self.current_diffs = diffs;
                } else {
                    self.current_diffs.clear();
                }
            }
            ActiveTab::History => {
                if let Some(commit) = self.commits.get(self.selected_commit_index) {
                    self.current_diffs = self.git.diff_commit(&commit.sha).unwrap_or_default();
                } else {
                    self.current_diffs.clear();
                }
            }
        }
        Ok(())
    }

    pub fn selected_file(&self) -> Option<&FileChange> {
        self.status.files.get(self.selected_file_index)
    }

    pub fn toggle_stage_selected(&mut self) -> anyhow::Result<()> {
        if let Some(file) = self.selected_file() {
            let path = file.path.clone();
            if file.is_staged() {
                self.git.unstage_file(&path)?;
                self.set_toast(format!("Unstaged {path}"), false);
            } else {
                self.git.stage_file(&path)?;
                self.set_toast(format!("Staged {path}"), false);
            }
            self.refresh_status()?;
        }
        Ok(())
    }

    pub fn toggle_stage_all(&mut self) -> anyhow::Result<()> {
        let any_unstaged = self.status.files.iter().any(|f| f.is_unstaged());
        if any_unstaged {
            self.git.stage_all()?;
            self.set_toast("Staged all changes".to_string(), false);
        } else {
            self.git.unstage_all()?;
            self.set_toast("Unstaged all changes".to_string(), false);
        }
        self.refresh_status()?;
        Ok(())
    }

    pub fn discard_selected_file(&mut self) -> anyhow::Result<()> {
        if let Some(file) = self.selected_file() {
            let path = file.path.clone();
            self.git.discard_file(&path)?;
            self.set_toast(format!("Discarded changes in {path}"), false);
            self.refresh_status()?;
        }
        Ok(())
    }

    pub fn commit(&mut self) -> anyhow::Result<()> {
        if self.commit_summary.trim().is_empty() {
            self.set_toast("Cannot commit: summary is required".to_string(), true);
            return Ok(());
        }

        let desc = if self.commit_description.trim().is_empty() {
            None
        } else {
            Some(self.commit_description.as_str())
        };

        match self
            .git
            .commit(&self.commit_summary, desc, &self.commit_co_authors)
        {
            Ok(sha) => {
                let short_sha = &sha[..7.min(sha.len())];
                self.set_toast(format!("Committed: {short_sha}"), false);
                self.commit_summary.clear();
                self.commit_description.clear();
                self.commit_co_authors.clear();
                self.refresh_status()?;
                self.load_commits(50)?;
                self.focused_pane = FocusedPane::FileList;
            }
            Err(e) => {
                self.set_toast(format!("Commit failed: {e}"), true);
            }
        }
        Ok(())
    }

    pub fn undo_commit(&mut self) -> anyhow::Result<()> {
        match self.git.undo_commit() {
            Ok(()) => {
                self.set_toast("Undid last commit (soft reset HEAD~1)".to_string(), false);
                self.refresh_status()?;
                self.load_commits(50)?;
            }
            Err(e) => {
                self.set_toast(format!("Undo failed: {e}"), true);
            }
        }
        Ok(())
    }

    pub fn sync(&mut self) -> anyhow::Result<()> {
        self.set_toast("Syncing with remote...".to_string(), false);
        let res = self.git.fetch(None);
        if let Err(e) = res {
            self.set_toast(format!("Fetch failed: {e}"), true);
            return Ok(());
        }

        let status = self.git.status()?;
        if status.behind > 0 {
            if let Err(e) = self.git.pull(None, None, false) {
                self.set_toast(format!("Pull failed: {e}"), true);
                return Ok(());
            }
        }
        if status.ahead > 0 {
            if let Err(e) = self.git.push(None, None, false) {
                self.set_toast(format!("Push failed: {e}"), true);
                return Ok(());
            }
        }

        self.refresh_status()?;
        self.set_toast("Repository synchronized".to_string(), false);
        Ok(())
    }

    pub fn open_branch_modal(&mut self) -> anyhow::Result<()> {
        self.branches = self.git.branches()?;
        self.selected_branch_index = self.branches.iter().position(|b| b.is_current).unwrap_or(0);
        self.branch_filter.clear();
        self.show_branch_modal = true;
        self.focused_pane = FocusedPane::BranchModal;
        Ok(())
    }

    pub fn checkout_selected_branch(&mut self) -> anyhow::Result<()> {
        let filtered = self.filtered_branches();
        if let Some(b) = filtered.get(self.selected_branch_index) {
            let name = b.name.clone();
            match self.git.checkout(&name) {
                Ok(()) => {
                    self.set_toast(format!("Switched to branch {name}"), false);
                    self.show_branch_modal = false;
                    self.focused_pane = FocusedPane::FileList;
                    self.refresh_status()?;
                    self.load_commits(50)?;
                }
                Err(e) => {
                    self.set_toast(format!("Checkout failed: {e}"), true);
                }
            }
        }
        Ok(())
    }

    pub fn filtered_branches(&self) -> Vec<&Branch> {
        if self.branch_filter.is_empty() {
            self.branches.iter().collect()
        } else {
            self.branches
                .iter()
                .filter(|b| {
                    b.name
                        .to_lowercase()
                        .contains(&self.branch_filter.to_lowercase())
                })
                .collect()
        }
    }

    pub fn set_toast(&mut self, message: impl Into<String>, is_error: bool) {
        self.status_message = Some((message.into(), is_error));
    }
}
