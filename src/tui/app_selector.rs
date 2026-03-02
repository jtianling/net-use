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
use crate::types::{AppInfo, AppMonitorState};

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

    fn monitor_state_label(app: &AppInfo) -> &'static str {
        match app.monitor_state {
            AppMonitorState::Monitoring => "MONITORING",
            AppMonitorState::Paused => "PAUSED",
            AppMonitorState::Unmonitored => "IDLE",
        }
    }

    fn monitor_state_style(state: AppMonitorState) -> Style {
        match state {
            AppMonitorState::Monitoring => Style::default().fg(Color::Green),
            AppMonitorState::Paused => Style::default().fg(Color::Yellow),
            AppMonitorState::Unmonitored => Style::default().fg(Color::DarkGray),
        }
    }

    fn row_line(app: &AppInfo) -> Line<'static> {
        let status = if app.pid.is_some() {
            Span::styled("● ", Style::default().fg(Color::Green))
        } else {
            Span::styled("○ ", Style::default().fg(Color::DarkGray))
        };

        let name = Span::styled(app.display_name.clone(), Style::default().fg(Color::White));
        let source = Span::styled(
            format!(" [{}]", Self::source_label(app)),
            Style::default().fg(Color::Yellow),
        );
        let monitor = Span::styled(
            format!(" [{}]", Self::monitor_state_label(app)),
            Self::monitor_state_style(app.monitor_state),
        );
        let bundle = Span::styled(
            format!("  {}", app.bundle_id.as_deref().unwrap_or("--")),
            Style::default().fg(Color::DarkGray),
        );
        let pid_text = match app.pid {
            Some(pid) => Span::styled(format!("  PID {pid}"), Style::default().fg(Color::Cyan)),
            None => Span::raw(""),
        };

        Line::from(vec![status, name, source, monitor, bundle, pid_text])
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
                ListItem::new(Self::row_line(app))
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

    pub fn run_with_tick(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
        mut on_tick: impl FnMut(),
    ) -> Result<Option<AppInfo>> {
        loop {
            on_tick();
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

#[cfg(test)]
mod tests {
    use super::AppSelector;
    use crate::types::{AppInfo, AppMonitorState};

    fn test_app(state: AppMonitorState) -> AppInfo {
        AppInfo {
            display_name: "Chrome".to_string(),
            bundle_id: Some("com.google.Chrome".to_string()),
            executable_name: "Chrome".to_string(),
            app_path: Some("/Applications/Google Chrome.app".to_string()),
            pid: Some(1234),
            monitor_state: state,
        }
    }

    #[test]
    fn test_row_displays_monitoring_state_label() {
        let monitoring = AppSelector::row_line(&test_app(AppMonitorState::Monitoring)).to_string();
        let paused = AppSelector::row_line(&test_app(AppMonitorState::Paused)).to_string();
        let idle = AppSelector::row_line(&test_app(AppMonitorState::Unmonitored)).to_string();

        assert!(monitoring.contains("[MONITORING]"));
        assert!(paused.contains("[PAUSED]"));
        assert!(idle.contains("[IDLE]"));
    }

    #[test]
    fn test_filter_behavior_unchanged_with_monitor_state() {
        let mut selector = AppSelector::new(vec![
            test_app(AppMonitorState::Monitoring),
            AppInfo {
                display_name: "curl".to_string(),
                bundle_id: None,
                executable_name: "curl".to_string(),
                app_path: None,
                pid: Some(42),
                monitor_state: AppMonitorState::Unmonitored,
            },
        ]);

        selector.filter_text = "cur".to_string();
        selector.apply_filter();

        assert_eq!(selector.filtered.len(), 1);
        assert_eq!(selector.filtered[0], 1);
    }
}
