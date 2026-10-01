use crate::app::{ActiveTab, App, FocusedPane};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(app: &mut App, key: KeyEvent) -> anyhow::Result<()> {
    if app.show_branch_modal {
        return handle_branch_modal_key(app, key);
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Char('q')
            if app.focused_pane != FocusedPane::CommitSummary
                && app.focused_pane != FocusedPane::CommitDescription =>
        {
            app.should_quit = true;
        }
        KeyCode::Tab => {
            app.focused_pane = match app.focused_pane {
                FocusedPane::FileList => FocusedPane::CommitSummary,
                FocusedPane::CommitSummary => FocusedPane::CommitDescription,
                FocusedPane::CommitDescription => FocusedPane::DiffViewer,
                FocusedPane::DiffViewer => FocusedPane::FileList,
                FocusedPane::BranchModal => FocusedPane::FileList,
            };
        }
        KeyCode::BackTab => {
            app.focused_pane = match app.focused_pane {
                FocusedPane::FileList => FocusedPane::DiffViewer,
                FocusedPane::CommitSummary => FocusedPane::FileList,
                FocusedPane::CommitDescription => FocusedPane::CommitSummary,
                FocusedPane::DiffViewer => FocusedPane::CommitDescription,
                FocusedPane::BranchModal => FocusedPane::FileList,
            };
        }
        KeyCode::Char('1')
            if app.focused_pane != FocusedPane::CommitSummary
                && app.focused_pane != FocusedPane::CommitDescription =>
        {
            app.active_tab = ActiveTab::Changes;
            app.load_selected_file_diff().ok();
        }
        KeyCode::Char('2')
            if app.focused_pane != FocusedPane::CommitSummary
                && app.focused_pane != FocusedPane::CommitDescription =>
        {
            app.active_tab = ActiveTab::History;
            app.load_selected_file_diff().ok();
        }
        KeyCode::Char('b')
            if app.focused_pane != FocusedPane::CommitSummary
                && app.focused_pane != FocusedPane::CommitDescription =>
        {
            app.open_branch_modal().ok();
        }
        KeyCode::Char('s')
            if app.focused_pane != FocusedPane::CommitSummary
                && app.focused_pane != FocusedPane::CommitDescription =>
        {
            app.sync().ok();
        }
        KeyCode::Char('u')
            if app.active_tab == ActiveTab::History
                && app.focused_pane != FocusedPane::CommitSummary
                && app.focused_pane != FocusedPane::CommitDescription =>
        {
            app.undo_commit().ok();
        }
        _ => match app.focused_pane {
            FocusedPane::FileList => handle_file_list_key(app, key)?,
            FocusedPane::CommitSummary => handle_commit_summary_key(app, key)?,
            FocusedPane::CommitDescription => handle_commit_desc_key(app, key)?,
            FocusedPane::DiffViewer => handle_diff_viewer_key(app, key)?,
            FocusedPane::BranchModal => handle_branch_modal_key(app, key)?,
        },
    }

    Ok(())
}

fn handle_file_list_key(app: &mut App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => match app.active_tab {
            ActiveTab::Changes => {
                if app.selected_file_index > 0 {
                    app.selected_file_index -= 1;
                    app.load_selected_file_diff().ok();
                }
            }
            ActiveTab::History => {
                if app.selected_commit_index > 0 {
                    app.selected_commit_index -= 1;
                    app.load_selected_file_diff().ok();
                }
            }
        },
        KeyCode::Down | KeyCode::Char('j') => match app.active_tab {
            ActiveTab::Changes => {
                if !app.status.files.is_empty()
                    && app.selected_file_index < app.status.files.len() - 1
                {
                    app.selected_file_index += 1;
                    app.load_selected_file_diff().ok();
                }
            }
            ActiveTab::History => {
                if !app.commits.is_empty() && app.selected_commit_index < app.commits.len() - 1 {
                    app.selected_commit_index += 1;
                    app.load_selected_file_diff().ok();
                }
            }
        },
        KeyCode::Char(' ') if app.active_tab == ActiveTab::Changes => {
            app.toggle_stage_selected().ok();
        }
        KeyCode::Char('a') if app.active_tab == ActiveTab::Changes => {
            app.toggle_stage_all().ok();
        }
        KeyCode::Char('d') if app.active_tab == ActiveTab::Changes => {
            app.discard_selected_file().ok();
        }
        KeyCode::Char('c') => {
            app.focused_pane = FocusedPane::CommitSummary;
        }
        _ => {}
    }
    Ok(())
}

fn handle_commit_summary_key(app: &mut App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Enter => {
            app.commit().ok();
        }
        KeyCode::Esc => {
            app.focused_pane = FocusedPane::FileList;
        }
        KeyCode::Backspace => {
            app.commit_summary.pop();
        }
        KeyCode::Char(c) => {
            app.commit_summary.push(c);
        }
        _ => {}
    }
    Ok(())
}

fn handle_commit_desc_key(app: &mut App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Enter => {
            app.commit().ok();
        }
        KeyCode::Esc => {
            app.focused_pane = FocusedPane::FileList;
        }
        KeyCode::Backspace => {
            app.commit_description.pop();
        }
        KeyCode::Char(c) => {
            app.commit_description.push(c);
        }
        _ => {}
    }
    Ok(())
}

fn handle_diff_viewer_key(app: &mut App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.diff_scroll > 0 {
                app.diff_scroll -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.diff_scroll += 1;
        }
        KeyCode::PageUp => {
            app.diff_scroll = app.diff_scroll.saturating_sub(15);
        }
        KeyCode::PageDown => {
            app.diff_scroll = app.diff_scroll.saturating_add(15);
        }
        KeyCode::Esc => {
            app.focused_pane = FocusedPane::FileList;
        }
        _ => {}
    }
    Ok(())
}

fn handle_branch_modal_key(app: &mut App, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.show_branch_modal = false;
            app.focused_pane = FocusedPane::FileList;
        }
        KeyCode::Up => {
            if app.selected_branch_index > 0 {
                app.selected_branch_index -= 1;
            }
        }
        KeyCode::Down => {
            let max_idx = app.filtered_branches().len().saturating_sub(1);
            if app.selected_branch_index < max_idx {
                app.selected_branch_index += 1;
            }
        }
        KeyCode::Enter => {
            app.checkout_selected_branch().ok();
        }
        KeyCode::Backspace => {
            app.branch_filter.pop();
            app.selected_branch_index = 0;
        }
        KeyCode::Char(c) => {
            app.branch_filter.push(c);
            app.selected_branch_index = 0;
        }
        _ => {}
    }
    Ok(())
}
