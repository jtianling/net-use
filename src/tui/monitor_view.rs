use std::collections::HashSet;
use std::io::{self, Write as _};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tokio::sync::mpsc;

use crate::tui::event::{AppEvent, is_quit, poll_event};
use crate::types::{AppInfo, DiscoveredAddress, MonitorEvent, ProcessInfo};

pub struct MonitorView {
    app_info: AppInfo,
    processes: Vec<ProcessInfo>,
    ipv4_subnets: Vec<String>,
    ipv6_addresses: Vec<String>,
    seen_addresses: HashSet<String>,
    start_time: Instant,
    last_new_ip: Option<Instant>,
    status_message: Option<(String, Instant)>,
    target_active: bool,
}

impl MonitorView {
    pub fn new(app_info: AppInfo) -> Self {
        Self {
            app_info,
            processes: Vec::new(),
            ipv4_subnets: Vec::new(),
            ipv6_addresses: Vec::new(),
            seen_addresses: HashSet::new(),
            start_time: Instant::now(),
            last_new_ip: None,
            status_message: None,
            target_active: false,
        }
    }

    pub fn restore_data(&mut self, ipv4: &[String], ipv6: &[String]) {
        for s in ipv4 {
            self.seen_addresses.insert(s.clone());
            self.ipv4_subnets.push(s.clone());
        }
        for s in ipv6 {
            self.seen_addresses.insert(s.clone());
            self.ipv6_addresses.push(s.clone());
        }
    }

    pub fn ipv4_data(&self) -> &[String] {
        &self.ipv4_subnets
    }

    pub fn ipv6_data(&self) -> &[String] {
        &self.ipv6_addresses
    }

    fn handle_event(&mut self, event: MonitorEvent) {
        match event {
            MonitorEvent::NewAddress(addr) => {
                let addr_str = addr.to_string();
                if self.seen_addresses.contains(&addr_str) {
                    return;
                }
                self.seen_addresses.insert(addr_str.clone());
                self.last_new_ip = Some(Instant::now());
                match &addr {
                    DiscoveredAddress::Ipv4Subnet(_) => {
                        self.ipv4_subnets.push(addr_str);
                    }
                    DiscoveredAddress::Ipv6Full(_) => {
                        self.ipv6_addresses.push(addr_str);
                    }
                }
            }
            MonitorEvent::ProcessAdded(info) => {
                if !self.processes.iter().any(|p| p.pid == info.pid) {
                    self.processes.push(info);
                }
            }
            MonitorEvent::ProcessRemoved(pid) => {
                self.processes.retain(|p| p.pid != pid);
            }
            MonitorEvent::TargetFound => {
                self.target_active = true;
            }
            MonitorEvent::TargetLost => {
                self.target_active = false;
                self.processes.clear();
            }
        }
    }

    fn export_to_file(&mut self) -> Result<String> {
        let filename = format!("net-use-export-{}.txt", chrono_timestamp());
        let mut file = std::fs::File::create(&filename)?;
        for s in &self.ipv4_subnets {
            writeln!(file, "{s}")?;
        }
        for s in &self.ipv6_addresses {
            writeln!(file, "{s}")?;
        }
        Ok(filename)
    }

    fn copy_to_clipboard(&self) -> Result<()> {
        let mut content = String::new();
        for s in &self.ipv4_subnets {
            content.push_str(s);
            content.push('\n');
        }
        for s in &self.ipv6_addresses {
            content.push_str(s);
            content.push('\n');
        }

        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Min(4),
                Constraint::Min(4),
                Constraint::Length(2),
            ])
            .split(frame.area());

        self.render_header(frame, chunks[0]);
        self.render_processes(frame, chunks[1]);
        self.render_ipv4(frame, chunks[2]);
        self.render_ipv6(frame, chunks[3]);
        self.render_status_bar(frame, chunks[4]);
    }

    fn render_header(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let elapsed = self.start_time.elapsed();
        let minutes = elapsed.as_secs() / 60;
        let seconds = elapsed.as_secs() % 60;

        let status_indicator = if self.target_active {
            Span::styled("● Monitoring", Style::default().fg(Color::Green))
        } else {
            Span::styled("○ Waiting", Style::default().fg(Color::Yellow))
        };

        let header_text = Line::from(vec![
            Span::styled("Target: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &self.app_info.display_name,
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("  ({})", self.app_info.bundle_id.as_deref().unwrap_or("--")),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("    "),
            status_indicator,
            Span::styled(
                format!(
                    "    Procs: {}    Uptime: {:02}:{:02}",
                    self.processes.len(),
                    minutes,
                    seconds
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        let header = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL).title(" net-use "));
        frame.render_widget(header, area);
    }

    fn render_processes(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let items: Vec<ListItem> = self
            .processes
            .iter()
            .take(4)
            .map(|p| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("PID {:<6}", p.pid),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(&p.name, Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let remaining = if self.processes.len() > 4 {
            format!(" Tracked Processes (+{} more) ", self.processes.len() - 4)
        } else {
            " Tracked Processes ".to_string()
        };

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(remaining));
        frame.render_widget(list, area);
    }

    fn render_ipv4(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let items: Vec<ListItem> = self
            .ipv4_subnets
            .iter()
            .map(|s| ListItem::new(Span::styled(s.as_str(), Style::default().fg(Color::White))))
            .collect();

        let title = format!(" IPv4 Subnets ({}) ", self.ipv4_subnets.len());
        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(list, area);
    }

    fn render_ipv6(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let items: Vec<ListItem> = self
            .ipv6_addresses
            .iter()
            .map(|s| ListItem::new(Span::styled(s.as_str(), Style::default().fg(Color::White))))
            .collect();

        let title = format!(" IPv6 Addresses ({}) ", self.ipv6_addresses.len());
        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(list, area);
    }

    fn render_status_bar(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let total = self.ipv4_subnets.len() + self.ipv6_addresses.len();

        let stability = match self.last_new_ip {
            Some(last) => {
                let elapsed = last.elapsed();
                if elapsed.as_secs() >= 60 {
                    format!("No new IPs for {} min", elapsed.as_secs() / 60)
                } else {
                    format!("Last new: {}s ago", elapsed.as_secs())
                }
            }
            None => "No IPs discovered yet".to_string(),
        };

        let status_msg = self
            .status_message
            .as_ref()
            .filter(|(_, t)| t.elapsed() < Duration::from_secs(5))
            .map(|(msg, _)| format!("  {msg}"))
            .unwrap_or_default();

        let line1 = Line::from(vec![
            Span::styled(
                format!(" Total: {total} "),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("│ {stability}"),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(status_msg, Style::default().fg(Color::Green)),
        ]);
        let line2 = Line::from(vec![Span::styled(
            " [E]xport  [C]opy  [Esc]Back  [Q]uit",
            Style::default().fg(Color::DarkGray),
        )]);

        let paragraph = Paragraph::new(vec![line1, line2]);
        frame.render_widget(paragraph, area);
    }

    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
        rx: &mut mpsc::UnboundedReceiver<MonitorEvent>,
    ) -> Result<MonitorAction> {
        loop {
            while let Ok(event) = rx.try_recv() {
                self.handle_event(event);
            }

            terminal.draw(|f| self.render(f))?;

            if let Some(evt) = poll_event(Duration::from_millis(50))? {
                match evt {
                    AppEvent::Key(key) => {
                        if is_quit(&key) {
                            return Ok(MonitorAction::Quit);
                        }
                        match key.code {
                            KeyCode::Esc => return Ok(MonitorAction::Back),
                            KeyCode::Char('e') | KeyCode::Char('E') => {
                                match self.export_to_file() {
                                    Ok(filename) => {
                                        self.status_message = Some((
                                            format!("Exported to {filename}"),
                                            Instant::now(),
                                        ));
                                    }
                                    Err(e) => {
                                        self.status_message =
                                            Some((format!("Export failed: {e}"), Instant::now()));
                                    }
                                }
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                match self.copy_to_clipboard() {
                                    Ok(()) => {
                                        self.status_message = Some((
                                            "Copied to clipboard".to_string(),
                                            Instant::now(),
                                        ));
                                    }
                                    Err(e) => {
                                        self.status_message =
                                            Some((format!("Copy failed: {e}"), Instant::now()));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    AppEvent::Tick => {}
                }
            }
        }
    }
}

pub enum MonitorAction {
    Quit,
    Back,
}

fn chrono_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}
