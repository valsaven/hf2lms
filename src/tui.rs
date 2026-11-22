use crate::model::{LinkStatus, Model, format_size};
use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        if let Err(err) = enable_raw_mode() {
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
            return Err(err.into());
        }
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
        let _ = disable_raw_mode();
    }
}

pub(crate) fn run_tui(models: &[Model], force: bool) -> Result<Option<Vec<usize>>> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut selected = vec![false; models.len()];
    let mut list_state = ListState::default();
    if !models.is_empty() {
        list_state.select(Some(0));
    }

    let result = loop {
        let selected_count = selected.iter().filter(|&&s| s).count();
        let conflict_count = models
            .iter()
            .filter(|m| matches!(m.status, LinkStatus::Conflict))
            .count();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(f.area());

            let items: Vec<ListItem> = models
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let prefix = match m.status {
                        LinkStatus::NotLinked => {
                            if selected[i] {
                                "[x]"
                            } else {
                                "[ ]"
                            }
                        }
                        LinkStatus::Linked => "[=]",
                        LinkStatus::Conflict => "[!]",
                    };
                    let text = format!("{prefix} {}  ({})", m.display, format_size(m.size));
                    let style = match m.status {
                        LinkStatus::Linked => Style::default().fg(Color::DarkGray),
                        LinkStatus::Conflict => Style::default().fg(Color::Yellow),
                        LinkStatus::NotLinked => {
                            if selected[i] {
                                Style::default().fg(Color::Green)
                            } else {
                                Style::default()
                            }
                        }
                    };
                    ListItem::new(text).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("hf2lms | Up/Down or j/k: move | Space: toggle | Enter: import | q: quit")
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("> ");

            f.render_stateful_widget(list, chunks[0], &mut list_state);

            let mut status = format!("{selected_count} selected");
            if conflict_count > 0 && !force {
                status.push_str(&format!(
                    " | {conflict_count} conflict(s) - rerun with --force to allow overwrite"
                ));
            } else if force && conflict_count > 0 {
                status.push_str(&format!(" | {conflict_count} conflict(s) - --force active"));
            }
            let status = Paragraph::new(status)
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(status, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let len = models.len();
            let cursor = list_state.selected().unwrap_or(0);

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break None,
                KeyCode::Up | KeyCode::Char('k') if len > 0 => {
                    list_state.select(Some((cursor + len - 1) % len));
                }
                KeyCode::Down | KeyCode::Char('j') if len > 0 => {
                    list_state.select(Some((cursor + 1) % len));
                }
                KeyCode::Char(' ') => {
                    if let Some(i) = list_state.selected() {
                        let can_toggle = match models[i].status {
                            LinkStatus::NotLinked => true,
                            LinkStatus::Conflict => force,
                            LinkStatus::Linked => false,
                        };
                        if can_toggle {
                            selected[i] = !selected[i];
                        }
                    }
                }
                KeyCode::Enter => {
                    let chosen: Vec<usize> = selected
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &s)| s.then_some(i))
                        .collect();
                    break Some(chosen);
                }
                _ => {}
            }
        }
    };

    // terminal restored by guard Drop before we return
    Ok(result)
}
