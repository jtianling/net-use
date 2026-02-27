use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::tui::event::{AppEvent, is_quit, poll_event};
use crate::types::AppInfo;

pub struct AppSelector {
    apps: Vec<AppInfo>,
    filtered: Vec<usize>,
    filter_text: String,
    list_state: ListState,
}

impl AppSelector {
    pub fn new(apps: Vec<AppInfo>) -> Self {
        let filtered: Vec<usize> = (0..apps.len()).collect();
        let mut list_state = ListState::default();
        if !filtered.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            apps,
            filtered,
            filter_text: String::new(),
            list_state,
        }
    }

    fn apply_filter(&mut self) {
        let query = self.filter_text.to_lowercase();
        self.filtered = self
            .apps
            .iter()
            .enumerate()
            .filter(|(_, app)| {
                if query.is_empty() {
                    return true;
                }

                let pid_matches = app
                    .pid
                    .map(|pid| pid.to_string().contains(&query))
                    .unwrap_or(false);
                app.display_name.to_lowercase().contains(&query)
                    || app
                        .bundle_id
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&query)
                    || pid_matches
            })
            .map(|(i, _)| i)
            .collect();

        if self.filtered.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    fn source_label(app: &AppInfo) -> &'static str {
        match (app.pid.is_some(), app.bundle_id.is_some()) {
            (true, true) => "GUI",
            (true, false) => "CLI",
            (false, true) => "APP",
            (false, false) => "UNK",
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<SelectorAction> {
        if is_quit(&key) {
            return Some(SelectorAction::Quit);
        }

        match key.code {
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected()
                    && let Some(&app_idx) = self.filtered.get(selected)
                {
                    return Some(SelectorAction::Selected(self.apps[app_idx].clone()));
                }
            }
            KeyCode::Up => {
                if let Some(selected) = self.list_state.selected()
                    && selected > 0
                {
                    self.list_state.select(Some(selected - 1));
                }
            }
            KeyCode::Down => {
                if let Some(selected) = self.list_state.selected()
                    && selected + 1 < self.filtered.len()
                {
                    self.list_state.select(Some(selected + 1));
                }
            }
            KeyCode::Backspace => {
                self.filter_text.pop();
                self.apply_filter();
            }
            KeyCode::Char(c) => {
                self.filter_text.push(c);
                self.apply_filter();
            }
            KeyCode::Esc => {
                if !self.filter_text.is_empty() {
                    self.filter_text.clear();
                    self.apply_filter();
                }
            }
            _ => {}
        }
        None
    }

    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(frame.area());

        let filter_display = if self.filter_text.is_empty() {
            "Type to filter...".to_string()
        } else {
            self.filter_text.clone()
        };
        let filter_style = if self.filter_text.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let filter = Paragraph::new(filter_display)
            .style(filter_style)
            .block(Block::default().borders(Borders::ALL).title(" Filter "));
        frame.render_widget(filter, chunks[0]);

        let items: Vec<ListItem> = self
            .filtered
            .iter()
            .map(|&idx| {
                let app = &self.apps[idx];
                let status = if app.pid.is_some() {
                    Span::styled("● ", Style::default().fg(Color::Green))
                } else {
                    Span::styled("○ ", Style::default().fg(Color::DarkGray))
                };

                let name = Span::styled(&app.display_name, Style::default().fg(Color::White));
                let source = Span::styled(
                    format!(" [{}]", Self::source_label(app)),
                    Style::default().fg(Color::Yellow),
                );

                let bundle = Span::styled(
                    format!("  {}", app.bundle_id.as_deref().unwrap_or("--")),
                    Style::default().fg(Color::DarkGray),
                );

                let pid_text = match app.pid {
                    Some(pid) => {
                        Span::styled(format!("  PID {pid}"), Style::default().fg(Color::Cyan))
                    }
                    None => Span::raw(""),
                };

                ListItem::new(Line::from(vec![status, name, source, bundle, pid_text]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Select Application "),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);

        let help = Paragraph::new(" ↑↓ Navigate  Enter Select  Q Quit  Type to filter")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    ) -> Result<Option<AppInfo>> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Some(evt) = poll_event(Duration::from_millis(100))? {
                match evt {
                    AppEvent::Key(key) => {
                        if let Some(action) = self.handle_key(key) {
                            return match action {
                                SelectorAction::Quit => Ok(None),
                                SelectorAction::Selected(app) => Ok(Some(app)),
                            };
                        }
                    }
                    AppEvent::Tick => {}
                }
            }
        }
    }
}

enum SelectorAction {
    Quit,
    Selected(AppInfo),
}
