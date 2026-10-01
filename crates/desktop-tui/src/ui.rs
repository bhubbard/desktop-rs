use crate::app::{ActiveTab, App, FocusedPane};
use desktop_core::DiffLineType;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Vertical split: Header, Main Body, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top Toolbar
            Constraint::Min(10),   // Main Area
            Constraint::Length(2), // Help / Status Bar
        ])
        .split(size);

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    if app.show_branch_modal {
        draw_branch_modal(f, app, size);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Repo info
            Constraint::Percentage(40), // Branch info
            Constraint::Percentage(30), // Sync status
        ])
        .split(area);

    let repo_name = app.git.repo_name();
    let repo_p = Paragraph::new(Line::from(vec![
        Span::styled(" 📦 ", Style::default().fg(Color::Yellow)),
        Span::styled(repo_name, Style::default().add_modifier(Modifier::BOLD)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Current Repository "),
    );
    f.render_widget(repo_p, header_chunks[0]);

    let ahead_behind = if app.status.ahead > 0 || app.status.behind > 0 {
        format!(" (↑{} ↓{})", app.status.ahead, app.status.behind)
    } else {
        String::new()
    };

    let branch_spans = vec![
        Span::styled(" 🌿 ", Style::default().fg(Color::Cyan)),
        Span::styled(
            &app.status.branch,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            ahead_behind,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" [b] Switch", Style::default().fg(Color::DarkGray)),
    ];
    let branch_p = Paragraph::new(Line::from(branch_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Current Branch "),
    );
    f.render_widget(branch_p, header_chunks[1]);

    let sync_spans = vec![
        Span::styled(" 🔄 ", Style::default().fg(Color::Green)),
        Span::styled(
            "[s] Sync Remote",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    let sync_p = Paragraph::new(Line::from(sync_spans))
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Remote Action "),
        );
    f.render_widget(sync_p, header_chunks[2]);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Left sidebar: files / commits
            Constraint::Percentage(65), // Right main: diff view
        ])
        .split(area);

    draw_left_sidebar(f, app, main_chunks[0]);
    draw_diff_panel(f, app, main_chunks[1]);
}

fn draw_left_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(6),    // List of changes or history
            Constraint::Length(match app.active_tab {
                ActiveTab::Changes => 7, // Commit box
                ActiveTab::History => 0,
            }),
        ])
        .split(area);

    // Render Tabs
    let tab_titles = vec![
        format!("1. Changes ({})", app.status.total_changes()),
        format!("2. History ({})", app.commits.len()),
    ];
    let selected_tab = match app.active_tab {
        ActiveTab::Changes => 0,
        ActiveTab::History => 1,
    };
    let tabs = Tabs::new(tab_titles)
        .select(selected_tab)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );
    f.render_widget(tabs, sidebar_chunks[0]);

    match app.active_tab {
        ActiveTab::Changes => {
            draw_changes_list(f, app, sidebar_chunks[1]);
            draw_commit_box(f, app, sidebar_chunks[2]);
        }
        ActiveTab::History => {
            draw_history_list(f, app, sidebar_chunks[1]);
        }
    }
}

fn draw_changes_list(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::FileList;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .status
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let checkbox = if file.is_conflicted() {
                Span::styled(
                    "[!] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )
            } else if file.is_partially_staged() {
                Span::styled("[~] ", Style::default().fg(Color::Yellow))
            } else if file.is_staged() {
                Span::styled(
                    "[✓] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else if file.is_untracked() {
                Span::styled("[?] ", Style::default().fg(Color::Magenta))
            } else {
                Span::styled("[ ] ", Style::default().fg(Color::DarkGray))
            };

            let status_indicator = Span::styled(
                format!("{:<2}", file.summary_status().indicator()),
                match file.summary_status() {
                    desktop_core::FileStatusType::Added => Style::default().fg(Color::Green),
                    desktop_core::FileStatusType::Deleted => Style::default().fg(Color::Red),
                    desktop_core::FileStatusType::Modified => Style::default().fg(Color::Yellow),
                    desktop_core::FileStatusType::Conflicted => {
                        Style::default().fg(Color::LightRed)
                    }
                    _ => Style::default().fg(Color::DarkGray),
                },
            );

            let path_span = Span::raw(&file.path);

            let is_selected = i == app.selected_file_index;
            let mut style = Style::default();
            if is_selected {
                style = style
                    .bg(Color::Rgb(40, 44, 52))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD);
            }

            ListItem::new(Line::from(vec![checkbox, status_indicator, path_span])).style(style)
        })
        .collect();

    let title = if app.status.files.is_empty() {
        " Changed Files (Working Tree Clean) "
    } else {
        " Changed Files [Space: toggle, a: all] "
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(title),
    );
    f.render_widget(list, area);
}

fn draw_commit_box(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Summary input
            Constraint::Length(3), // Description input
            Constraint::Length(1), // Commit button
        ])
        .split(area);

    let summary_focused = app.focused_pane == FocusedPane::CommitSummary;
    let desc_focused = app.focused_pane == FocusedPane::CommitDescription;

    let summary_p = Paragraph::new(app.commit_summary.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if summary_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            })
            .title(" Commit Summary (required) "),
    );
    f.render_widget(summary_p, chunks[0]);

    let desc_p = Paragraph::new(app.commit_description.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if desc_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            })
            .title(" Description (optional) "),
    );
    f.render_widget(desc_p, chunks[1]);

    let btn_title = format!(" [c] Commit to {} ", app.status.branch);
    let commit_btn = Paragraph::new(btn_title)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(commit_btn, chunks[2]);
}

fn draw_history_list(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::FileList;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .commits
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let is_selected = i == app.selected_commit_index;
            let mut style = Style::default();
            if is_selected {
                style = style
                    .bg(Color::Rgb(40, 44, 52))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD);
            }

            let sha_span = Span::styled(
                format!("{:<8} ", commit.short_sha),
                Style::default().fg(Color::Yellow),
            );
            let summary_span = Span::raw(&commit.summary);

            ListItem::new(Line::from(vec![sha_span, summary_span])).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(" Commit History [u: undo last] "),
    );
    f.render_widget(list, area);
}

fn draw_diff_panel(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::DiffViewer;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let mut lines = Vec::new();

    if app.current_diffs.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            " No diff available for current selection",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        for diff in &app.current_diffs {
            let file_header = format!(
                "Diff: {} (+{} -{})",
                diff.file_path(),
                diff.added_count(),
                diff.deleted_count()
            );
            lines.push(Line::from(vec![Span::styled(
                file_header,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));

            if diff.is_binary {
                lines.push(Line::from(vec![Span::styled(
                    " [Binary file content cannot be viewed in unified diff]",
                    Style::default().fg(Color::Yellow),
                )]));
                continue;
            }

            for hunk in &diff.hunks {
                lines.push(Line::from(vec![Span::styled(
                    &hunk.header,
                    Style::default()
                        .fg(Color::Rgb(100, 160, 240))
                        .add_modifier(Modifier::ITALIC),
                )]));

                for line in &hunk.lines {
                    let old_no = line
                        .old_lineno
                        .map(|n| format!("{:>4} ", n))
                        .unwrap_or_else(|| "     ".to_string());
                    let new_no = line
                        .new_lineno
                        .map(|n| format!("{:>4} ", n))
                        .unwrap_or_else(|| "     ".to_string());

                    let num_span = Span::styled(
                        format!("{old_no}{new_no} │ "),
                        Style::default().fg(Color::Rgb(90, 95, 110)),
                    );

                    let (prefix, content_style) = match line.line_type {
                        DiffLineType::Addition => ("+", Style::default().fg(Color::Green)),
                        DiffLineType::Deletion => ("-", Style::default().fg(Color::Red)),
                        DiffLineType::Header => ("@", Style::default().fg(Color::Cyan)),
                        DiffLineType::NoNewline => ("\\", Style::default().fg(Color::DarkGray)),
                        DiffLineType::Context => (" ", Style::default().fg(Color::Gray)),
                    };

                    let content_span =
                        Span::styled(format!("{} {}", prefix, line.content), content_style);

                    lines.push(Line::from(vec![num_span, content_span]));
                }
            }
        }
    }

    let diff_p = Paragraph::new(lines)
        .scroll((app.diff_scroll, 0))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .title(" Changes Diff Viewer "),
        );
    f.render_widget(diff_p, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let status_text = if let Some((msg, is_err)) = &app.status_message {
        let style = if *is_err {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        };
        Span::styled(format!(" ℹ  {}", msg), style)
    } else {
        Span::styled(" Ready", Style::default().fg(Color::DarkGray))
    };
    f.render_widget(Paragraph::new(Line::from(vec![status_text])), chunks[0]);

    let help_spans = vec![
        Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
        Span::raw("Pane "),
        Span::styled("[Space] ", Style::default().fg(Color::Cyan)),
        Span::raw("Stage "),
        Span::styled("[c] ", Style::default().fg(Color::Cyan)),
        Span::raw("Commit "),
        Span::styled("[b] ", Style::default().fg(Color::Cyan)),
        Span::raw("Branch "),
        Span::styled("[s] ", Style::default().fg(Color::Cyan)),
        Span::raw("Sync "),
        Span::styled("[q] ", Style::default().fg(Color::Cyan)),
        Span::raw("Quit "),
    ];
    let help_p = Paragraph::new(Line::from(help_spans)).alignment(Alignment::Right);
    f.render_widget(help_p, chunks[1]);
}

fn draw_branch_modal(f: &mut Frame, app: &App, area: Rect) {
    let modal_area = centered_rect(60, 60, area);
    f.render_widget(Clear, modal_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter input
            Constraint::Min(5),    // Branch list
            Constraint::Length(2), // Help
        ])
        .split(modal_area);

    let filter_p = Paragraph::new(app.branch_filter.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Filter Branches "),
    );
    f.render_widget(filter_p, chunks[0]);

    let filtered = app.filtered_branches();
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let is_selected = i == app.selected_branch_index;
            let mut style = Style::default();
            if is_selected {
                style = style
                    .bg(Color::Rgb(40, 44, 52))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD);
            }

            let marker = if b.is_current {
                Span::styled("* ", Style::default().fg(Color::Green))
            } else {
                Span::raw("  ")
            };

            let name_span = Span::styled(
                &b.name,
                if b.is_remote {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                },
            );

            ListItem::new(Line::from(vec![marker, name_span])).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Branches "),
    );
    f.render_widget(list, chunks[1]);

    let help = Paragraph::new(" [Enter] Checkout  [Esc] Cancel ")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
