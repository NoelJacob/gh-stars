use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

use crate::app::{App, AppState};

pub async fn run(mut app: App) -> Result<()> {
    // Set up terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Load data
    app.load_data().await?;

    // Run event loop
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    res
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Handle keyboard events
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Loading => {}
                    AppState::RepoList => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Char('j') | KeyCode::Down => app.next_repo(),
                        KeyCode::Char('k') | KeyCode::Up => app.prev_repo(),
                        KeyCode::Char('l') | KeyCode::Enter => app.select_list_selector(),
                        KeyCode::Char('s') => {
                            app.get_ai_suggestion().await?;
                        }
                        _ => {}
                    },
                    AppState::ListSelect => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.select_repo_list(),
                        KeyCode::Char('j') | KeyCode::Down => app.next_list(),
                        KeyCode::Char('k') | KeyCode::Up => app.prev_list(),
                        KeyCode::Char('l') | KeyCode::Enter => app.confirm_action(),
                        _ => {}
                    },
                    AppState::Confirmation => match key.code {
                        KeyCode::Char('y') | KeyCode::Enter => {
                            match app.add_current_repo_to_list().await {
                                Ok(_) => app.select_repo_list(),
                                Err(e) => app.set_error(format!("Failed to add to list: {}", e)),
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Esc => app.select_list_selector(),
                        _ => {}
                    },
                    AppState::Error(_) => {
                        if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                            app.select_repo_list();
                        }
                    }
                }

                // Global shortcuts
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    app.should_quit = true;
                }
            }
        }

        // If we should quit, break the loop
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    // Title
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "GitHub Stars Manager",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ])])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    // Footer with help text
    let footer_text = match app.state {
        AppState::Loading => vec![Line::from("Loading...")],
        AppState::RepoList => vec![Line::from(vec![
            Span::raw("↑/↓: Navigate • "),
            Span::raw("Enter: Select • "),
            Span::styled("s: AI Suggestion", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" • q: Quit"),
        ])],
        AppState::ListSelect => vec![Line::from(
            "↑/↓: Navigate • Enter: Confirm • Esc: Back",
        )],
        AppState::Confirmation => vec![Line::from("y: Confirm • n: Cancel")],
        AppState::Error(_) => vec![Line::from("Press Enter to continue")],
    };

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);

    // Main content
    match app.state {
        AppState::Loading => {
            let loading = Paragraph::new("Loading data...")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Loading"));
            f.render_widget(loading, chunks[1]);
        }
        AppState::RepoList => {
            render_repo_list(f, app, chunks[1]);
        }
        AppState::ListSelect => {
            render_list_selector(f, app, chunks[1]);
        }
        AppState::Confirmation => {
            if let (Some(repo), Some(list)) = (
                app.starred_repos.get(app.selected_repo_idx),
                app.user_lists.get(app.selected_list_idx),
            ) {
                let confirm_text = format!(
                    "Add repository '{}' to list '{}'?",
                    repo.name, list.name
                );
                let confirm = Paragraph::new(confirm_text)
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title("Confirm"));
                f.render_widget(confirm, chunks[1]);
            }
        }
        AppState::Error(ref message) => {
            let error = Paragraph::new(message.clone())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Error")
                        .style(Style::default().fg(Color::Red)),
                );
            f.render_widget(error, chunks[1]);
        }
    }

    // Show message if available
    if let Some(message) = &app.message {
        let area = centered_rect(60, 20, f.size());
        let message_widget = Paragraph::new(message.clone())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Message")
                    .style(Style::default().fg(Color::Yellow)),
            );
        f.render_widget(message_widget, area);
    }
}

fn render_repo_list<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let repos: Vec<ListItem> = app
        .starred_repos
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let style = if i == app.selected_repo_idx {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let desc = repo
                .description
                .clone()
                .unwrap_or_else(|| "No description".to_string());
            let truncated_desc = if desc.len() > 60 {
                format!("{}...", &desc[..57])
            } else {
                desc
            };

            let content = vec![
                Line::from(vec![
                    Span::styled(&repo.name, style.add_modifier(Modifier::BOLD)),
                    Span::raw(" - "),
                    Span::styled(truncated_desc, style),
                ]),
            ];

            ListItem::new(content)
        })
        .collect();

    let repo_list = List::new(repos)
        .block(Block::default().borders(Borders::ALL).title("Starred Repositories"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(repo_list, area);
}

fn render_list_selector<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let lists: Vec<ListItem> = app
        .user_lists
        .iter()
        .enumerate()
        .map(|(i, list)| {
            let is_suggested = app
                .suggested_list
                .as_ref()
                .map_or(false, |sugg| sugg == &list.name);

            let mut style = if i == app.selected_list_idx {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let mut spans = vec![Span::styled(&list.name, style)];

            if is_suggested {
                style = style.fg(Color::Green);
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    "(AI Suggested)",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            if let Some(desc) = &list.description {
                spans.push(Span::raw(" - "));
                spans.push(Span::styled(desc, style));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list_title = if app.suggested_list.is_some() {
        "Select List (Green = AI Suggested)"
    } else {
        "Select List"
    };

    let list_selector = List::new(lists)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list_selector, area);
}

/// Helper function to create a centered rect using percentage values
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}