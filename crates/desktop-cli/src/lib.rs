use clap::{Parser, Subcommand};
use colored::Colorize;
use desktop_core::models::Author;
use desktop_git::GitClient;
use desktop_github::GitHubClient;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "desktop",
    author = "Brandon Hubbard <bhubbard@users.noreply.github.com>",
    version,
    about = "Blazingly fast pure-Rust GitHub Desktop CLI and TUI",
    long_about = "A high-performance, memory-safe, lightweight pure-Rust fork of GitHub Desktop.\nRuns everywhere with zero Electron bloat."
)]
pub struct Cli {
    /// Path to git repository (defaults to current working directory)
    #[arg(short = 'C', long, global = true)]
    pub path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch the interactive GitHub Desktop Terminal UI
    Tui,

    /// Launch the native Tauri GUI application (GitHub Desktop)
    Gui,

    /// Show repository status, branch, ahead/behind, and changed files
    Status,

    /// View colorized diff for working tree or staged changes
    Diff {
        /// Specific file path to diff
        file: Option<String>,

        /// View staged changes instead of unstaged
        #[arg(short, long)]
        staged: bool,
    },

    /// Stage files or all changes
    Stage {
        /// File paths to stage
        files: Vec<String>,

        /// Stage all changed files
        #[arg(short, long)]
        all: bool,
    },

    /// Unstage files or all changes
    Unstage {
        /// File paths to unstage
        files: Vec<String>,

        /// Unstage all staged files
        #[arg(short, long)]
        all: bool,
    },

    /// Discard local modifications to files
    Discard {
        /// File paths to discard
        files: Vec<String>,
    },

    /// Create a commit with summary, description, and co-authors
    Commit {
        /// Commit summary (first line)
        #[arg(short, long)]
        message: String,

        /// Extended commit description / body
        #[arg(short, long)]
        description: Option<String>,

        /// Co-authors (e.g. "Name <email@domain.com>")
        #[arg(long = "co-author")]
        co_authors: Vec<String>,
    },

    /// Safely undo the latest commit keeping changes staged
    Undo,

    /// Branch operations (list, switch, create, delete)
    Branch {
        /// Switch / checkout branch
        #[arg(short, long)]
        checkout: Option<String>,

        /// Create and checkout a new branch
        #[arg(short = 'b', long = "create")]
        create: Option<String>,

        /// Delete a branch
        #[arg(short, long)]
        delete: Option<String>,

        /// Force delete branch (-D)
        #[arg(short, long)]
        force: bool,
    },

    /// Synchronize repository with remote (fetch, pull, push)
    Sync {
        /// Only fetch from remote
        #[arg(long)]
        fetch_only: bool,

        /// Only pull changes from remote
        #[arg(long)]
        pull_only: bool,

        /// Only push commits to remote
        #[arg(long)]
        push_only: bool,
    },

    /// Stash management (save, pop, list, drop)
    Stash {
        #[command(subcommand)]
        action: Option<StashAction>,
    },

    /// GitHub Pull Request integration
    Pr {
        #[command(subcommand)]
        action: PrAction,
    },

    /// View visual commit history log
    Log {
        /// Number of commits to show (default: 15)
        #[arg(short = 'n', long, default_value = "15")]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum StashAction {
    /// Save changes to stash
    Save {
        /// Optional stash message
        message: Option<String>,
        /// Keep indexed changes
        #[arg(long)]
        keep_index: bool,
    },
    /// Pop the most recent stash or specified index
    Pop {
        /// Index of stash to pop
        index: Option<usize>,
    },
    /// List all stashes
    List,
    /// Drop a stash entry
    Drop {
        /// Index of stash to drop
        index: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum PrAction {
    /// List open pull requests on GitHub
    List {
        /// Filter by state: open, closed, all (default: open)
        #[arg(short, long, default_value = "open")]
        state: String,
    },
    /// Check out a pull request branch locally
    Checkout {
        /// Pull request number
        number: u64,
    },
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let target_dir = cli
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let git = match GitClient::open_or_find(&target_dir) {
        Ok(client) => client,
        Err(_) => {
            eprintln!(
                "{} Not a git repository: {}",
                "Error:".bright_red().bold(),
                target_dir.display()
            );
            std::process::exit(1);
        }
    };

    match cli.command {
        None => {
            // Default action: launch TUI if interactive terminal, otherwise show status
            use std::io::IsTerminal;
            if std::io::stdout().is_terminal() {
                desktop_tui::run_tui(git)?;
            } else {
                cmd_status(&git)?;
            }
        }
        Some(Commands::Tui) => {
            desktop_tui::run_tui(git)?;
        }
        Some(Commands::Gui) => {
            cmd_gui()?;
        }
        Some(Commands::Status) => {
            cmd_status(&git)?;
        }
        Some(Commands::Diff { file, staged }) => {
            cmd_diff(&git, file.as_deref(), staged)?;
        }
        Some(Commands::Stage { files, all }) => {
            cmd_stage(&git, &files, all)?;
        }
        Some(Commands::Unstage { files, all }) => {
            cmd_unstage(&git, &files, all)?;
        }
        Some(Commands::Discard { files }) => {
            cmd_discard(&git, &files)?;
        }
        Some(Commands::Commit {
            message,
            description,
            co_authors,
        }) => {
            cmd_commit(&git, &message, description.as_deref(), &co_authors)?;
        }
        Some(Commands::Undo) => {
            cmd_undo(&git)?;
        }
        Some(Commands::Branch {
            checkout,
            create,
            delete,
            force,
        }) => {
            cmd_branch(
                &git,
                checkout.as_deref(),
                create.as_deref(),
                delete.as_deref(),
                force,
            )?;
        }
        Some(Commands::Sync {
            fetch_only,
            pull_only,
            push_only,
        }) => {
            cmd_sync(&git, fetch_only, pull_only, push_only)?;
        }
        Some(Commands::Stash { action }) => {
            cmd_stash(&git, action)?;
        }
        Some(Commands::Pr { action }) => {
            cmd_pr(&git, action).await?;
        }
        Some(Commands::Log { limit }) => {
            cmd_log(&git, limit)?;
        }
    }

    Ok(())
}

fn cmd_status(git: &GitClient) -> anyhow::Result<()> {
    let status = git.status()?;
    let repo_name = git.repo_name();

    println!(
        "\n  {} {}",
        "📦 Repository:".bold(),
        repo_name.cyan().bold()
    );

    let ahead_behind = if status.ahead > 0 || status.behind > 0 {
        format!(" (↑{} ahead, ↓{} behind)", status.ahead, status.behind)
            .yellow()
            .to_string()
    } else {
        String::new()
    };

    println!(
        "  {} {}{}",
        "🌿 Branch:".bold(),
        status.branch.bright_cyan().bold(),
        ahead_behind
    );

    if status.is_clean() {
        println!(
            "\n  {} Working tree clean. Nothing to commit.\n",
            "✓".green().bold()
        );
        return Ok(());
    }

    let staged = status.staged_files();
    let unstaged = status.unstaged_files();

    if !staged.is_empty() {
        println!("\n  {}", "Staged Changes:".green().bold());
        for f in staged {
            let indicator = f.staged_status.indicator();
            println!(
                "    {} {} {}",
                "[✓]".green(),
                indicator.green().bold(),
                f.path
            );
        }
    }

    if !unstaged.is_empty() {
        println!("\n  {}", "Changes not staged for commit:".yellow().bold());
        for f in unstaged {
            let indicator = f.summary_status().indicator();
            let color_indicator = match f.summary_status() {
                desktop_core::FileStatusType::Untracked => "?".magenta(),
                desktop_core::FileStatusType::Deleted => "D".red(),
                desktop_core::FileStatusType::Conflicted => "!".bright_red().bold(),
                _ => indicator.yellow(),
            };
            let checkbox = if f.is_untracked() {
                "[?]".magenta()
            } else {
                "[ ]".yellow()
            };
            println!("    {} {} {}", checkbox, color_indicator.bold(), f.path);
        }
    }

    println!();
    Ok(())
}

fn cmd_diff(git: &GitClient, file: Option<&str>, staged: bool) -> anyhow::Result<()> {
    let diffs = if staged {
        git.diff_staged(file)?
    } else {
        git.diff_unstaged(file)?
    };

    if diffs.is_empty() {
        println!("No changes found.");
        return Ok(());
    }

    for diff in diffs {
        println!(
            "{}",
            format!(
                "--- Diff: {} (+{} -{}) ---",
                diff.file_path(),
                diff.added_count(),
                diff.deleted_count()
            )
            .cyan()
            .bold()
        );

        if diff.is_binary {
            println!("  {}", "[Binary file differs]".yellow());
            continue;
        }

        for hunk in diff.hunks {
            println!("{}", hunk.header.blue());
            for line in hunk.lines {
                match line.line_type {
                    desktop_core::DiffLineType::Addition => {
                        println!("{}", format!("+{}", line.content).green());
                    }
                    desktop_core::DiffLineType::Deletion => {
                        println!("{}", format!("-{}", line.content).red());
                    }
                    desktop_core::DiffLineType::Header => {
                        println!("{}", line.content.cyan());
                    }
                    desktop_core::DiffLineType::NoNewline => {
                        println!("{}", line.content.dimmed());
                    }
                    desktop_core::DiffLineType::Context => {
                        println!(" {}", line.content);
                    }
                }
            }
        }
        println!();
    }

    Ok(())
}

fn cmd_stage(git: &GitClient, files: &[String], all: bool) -> anyhow::Result<()> {
    if all || files.is_empty() {
        git.stage_all()?;
        println!("  {} Staged all changes", "✓".green());
    } else {
        for f in files {
            git.stage_file(f)?;
            println!("  {} Staged {}", "✓".green(), f);
        }
    }
    Ok(())
}

fn cmd_unstage(git: &GitClient, files: &[String], all: bool) -> anyhow::Result<()> {
    if all || files.is_empty() {
        git.unstage_all()?;
        println!("  {} Unstaged all changes", "✓".green());
    } else {
        for f in files {
            git.unstage_file(f)?;
            println!("  {} Unstaged {}", "✓".green(), f);
        }
    }
    Ok(())
}

fn cmd_discard(git: &GitClient, files: &[String]) -> anyhow::Result<()> {
    for f in files {
        git.discard_file(f)?;
        println!("  {} Discarded changes in {}", "✓".green(), f);
    }
    Ok(())
}

fn cmd_commit(
    git: &GitClient,
    summary: &str,
    description: Option<&str>,
    co_author_strings: &[String],
) -> anyhow::Result<()> {
    let mut co_authors = Vec::new();
    for s in co_author_strings {
        if let Some(author) = Author::parse_trailer(&format!("Co-authored-by: {s}")) {
            co_authors.push(author);
        } else if let Some(author) = Author::parse_trailer(s) {
            co_authors.push(author);
        }
    }

    let sha = git.commit(summary, description, &co_authors)?;
    let short_sha = &sha[..7.min(sha.len())];
    println!(
        "  {} Committed [{}] {}",
        "✓".green().bold(),
        short_sha.yellow().bold(),
        summary
    );
    if !co_authors.is_empty() {
        for ca in co_authors {
            println!("    {} Co-author: {} <{}>", "·".dimmed(), ca.name, ca.email);
        }
    }
    Ok(())
}

fn cmd_undo(git: &GitClient) -> anyhow::Result<()> {
    git.undo_commit()?;
    println!(
        "  {} Undid last commit (HEAD~1). Staged changes preserved.",
        "✓".green().bold()
    );
    Ok(())
}

fn cmd_branch(
    git: &GitClient,
    checkout: Option<&str>,
    create: Option<&str>,
    delete: Option<&str>,
    force: bool,
) -> anyhow::Result<()> {
    if let Some(new_branch) = create {
        git.create_branch(new_branch, None)?;
        println!(
            "  {} Created and switched to branch {}",
            "✓".green(),
            new_branch.cyan().bold()
        );
        return Ok(());
    }

    if let Some(branch_to_checkout) = checkout {
        git.checkout(branch_to_checkout)?;
        println!(
            "  {} Switched to branch {}",
            "✓".green(),
            branch_to_checkout.cyan().bold()
        );
        return Ok(());
    }

    if let Some(branch_to_delete) = delete {
        git.delete_branch(branch_to_delete, force)?;
        println!(
            "  {} Deleted branch {}",
            "✓".green(),
            branch_to_delete.cyan()
        );
        return Ok(());
    }

    let branches = git.branches()?;
    println!("\n  {}", "Branches:".bold());
    for b in branches {
        let prefix = if b.is_current {
            "* ".green().bold()
        } else {
            "  ".normal()
        };
        let name_styled = if b.is_remote {
            b.name.dimmed()
        } else if b.is_current {
            b.name.green().bold()
        } else {
            b.name.normal()
        };
        let upstream = b
            .upstream
            .map(|u| format!(" -> [{u}]").dimmed().to_string())
            .unwrap_or_default();
        println!("  {}{}{}", prefix, name_styled, upstream);
    }
    println!();

    Ok(())
}

fn cmd_sync(
    git: &GitClient,
    fetch_only: bool,
    pull_only: bool,
    push_only: bool,
) -> anyhow::Result<()> {
    if fetch_only {
        git.fetch(None)?;
        println!("  {} Fetched from origin", "✓".green());
        return Ok(());
    }
    if pull_only {
        let out = git.pull(None, None, false)?;
        println!("  {} Pull completed:\n{}", "✓".green(), out.trim());
        return Ok(());
    }
    if push_only {
        let out = git.push(None, None, false)?;
        println!("  {} Push completed:\n{}", "✓".green(), out.trim());
        return Ok(());
    }

    // Default: full sync (fetch, pull if behind, push if ahead)
    println!("  {} Fetching latest changes from remote...", "◈".yellow());
    git.fetch(None)?;

    let status = git.status()?;
    if status.behind > 0 {
        println!(
            "  {} Pulling {} behind commits...",
            "◈".yellow(),
            status.behind
        );
        git.pull(None, None, false)?;
    }
    if status.ahead > 0 {
        println!(
            "  {} Pushing {} ahead commits...",
            "◈".yellow(),
            status.ahead
        );
        git.push(None, None, false)?;
    }

    println!("  {} Synchronization complete.", "✓".green().bold());
    Ok(())
}

fn cmd_stash(git: &GitClient, action: Option<StashAction>) -> anyhow::Result<()> {
    match action {
        None | Some(StashAction::List) => {
            let stashes = git.stash_list()?;
            if stashes.is_empty() {
                println!("No stashes found.");
            } else {
                println!("\n  {}", "Stash entries:".bold());
                for s in stashes {
                    println!(
                        "    {} [{}] ({}) {}",
                        format!("stash@{{{}}}", s.index).yellow(),
                        s.sha[..7.min(s.sha.len())].dimmed(),
                        s.branch.cyan(),
                        s.message
                    );
                }
                println!();
            }
        }
        Some(StashAction::Save {
            message,
            keep_index,
        }) => {
            git.stash_save(message.as_deref(), keep_index)?;
            println!("  {} Stashed changes successfully", "✓".green());
        }
        Some(StashAction::Pop { index }) => {
            git.stash_pop(index)?;
            println!("  {} Popped stash successfully", "✓".green());
        }
        Some(StashAction::Drop { index }) => {
            git.stash_drop(index)?;
            println!("  {} Dropped stash@{{{}}}", "✓".green(), index);
        }
    }
    Ok(())
}

async fn cmd_pr(git: &GitClient, action: PrAction) -> anyhow::Result<()> {
    let remotes = git.run_git(&["remote", "get-url", "origin"]).ok();
    let (owner, repo) = if let Some(url) = remotes.as_deref() {
        GitHubClient::parse_owner_repo(url)
            .ok_or_else(|| anyhow::anyhow!("Could not parse GitHub repo from origin: {url}"))?
    } else {
        return Err(anyhow::anyhow!("No 'origin' remote found in repository"));
    };

    let client = GitHubClient::from_env();

    match action {
        PrAction::List { state } => {
            println!(
                "\n  {} Pull Requests for {}/{}:",
                "🐙".cyan(),
                owner.bold(),
                repo.bold()
            );
            let prs = client
                .list_pull_requests(&owner, &repo, Some(&state))
                .await?;
            if prs.is_empty() {
                println!("    No {} pull requests found.\n", state);
                return Ok(());
            }

            for pr in prs {
                let draft_marker = if pr.is_draft {
                    " [Draft]".dimmed()
                } else {
                    "".normal()
                };
                println!(
                    "    {} {} {}{} {}",
                    format!("#{}", pr.number).green().bold(),
                    pr.title,
                    draft_marker,
                    format!("({})", pr.head_branch).dimmed(),
                    format!("by @{}", pr.author).cyan()
                );
            }
            println!();
        }
        PrAction::Checkout { number } => {
            println!("  {} Fetching PR #{}...", "◈".yellow(), number);
            // Fetch refs/pull/ID/head into pr/ID
            let pr_ref = format!("refs/pull/{number}/head:pr/{number}");
            git.run_git(&["fetch", "origin", &pr_ref])?;
            let branch = format!("pr/{number}");
            git.checkout(&branch)?;
            println!(
                "  {} Checked out PR #{} onto branch {}",
                "✓".green().bold(),
                number,
                branch.cyan().bold()
            );
        }
    }

    Ok(())
}

fn cmd_log(git: &GitClient, limit: usize) -> anyhow::Result<()> {
    let commits = git.log(limit)?;
    if commits.is_empty() {
        println!("No commit history found.");
        return Ok(());
    }

    println!("\n  {}", "Commit History:".bold());
    for c in commits {
        println!(
            "  * {} {} {}",
            c.short_sha.yellow().bold(),
            c.summary.bold(),
            format!("({})", c.date).dimmed()
        );
        println!("    {} <{}>", c.author.name.cyan(), c.author.email.dimmed());
        for ca in c.co_authors {
            println!(
                "    {} Co-author: {} <{}>",
                "·".dimmed(),
                ca.name.green(),
                ca.email.dimmed()
            );
        }
        if let Some(body) = c.body {
            for b_line in body.lines().take(3) {
                println!("      {}", b_line.dimmed());
            }
        }
    }
    println!();
    Ok(())
}

fn cmd_gui() -> anyhow::Result<()> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let gui_bin = dir.join("desktop-gui");
            if gui_bin.exists() {
                let _ = std::process::Command::new(gui_bin).spawn()?;
                println!("  {} Launched GitHub Desktop GUI", "✓".green());
                return Ok(());
            }
        }
    }
    // Fallback to searching PATH
    let _ = std::process::Command::new("desktop-gui").spawn()?;
    println!("  {} Launched GitHub Desktop GUI", "✓".green());
    Ok(())
}

