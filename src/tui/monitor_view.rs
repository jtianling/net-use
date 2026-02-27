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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddressDisplayMode {
    Masked,
    Raw,
}

impl AddressDisplayMode {
    fn label(self) -> &'static str {
        match self {
            Self::Masked => "Masked",
            Self::Raw => "Raw",
        }
    }

    fn toggle(self) -> Self {
        match self {
            Self::Masked => Self::Raw,
            Self::Raw => Self::Masked,
        }
    }
}

pub struct MonitorView {
    app_info: AppInfo,
    processes: Vec<ProcessInfo>,
    ipv4_masked_subnets: Vec<String>,
    ipv4_raw_addresses: Vec<String>,
    ipv6_masked_addresses: Vec<String>,
    ipv6_raw_addresses: Vec<String>,
    seen_canonical_addresses: HashSet<String>,
    seen_ipv4_raw: HashSet<String>,
    seen_ipv6_raw: HashSet<String>,
    start_time: Instant,
    last_new_ip: Option<Instant>,
    status_message: Option<(String, Instant)>,
    target_active: bool,
    address_display_mode: AddressDisplayMode,
}

impl MonitorView {
    pub fn new(app_info: AppInfo) -> Self {
        Self {
            app_info,
            processes: Vec::new(),
            ipv4_masked_subnets: Vec::new(),
            ipv4_raw_addresses: Vec::new(),
            ipv6_masked_addresses: Vec::new(),
            ipv6_raw_addresses: Vec::new(),
            seen_canonical_addresses: HashSet::new(),
            seen_ipv4_raw: HashSet::new(),
            seen_ipv6_raw: HashSet::new(),
            start_time: Instant::now(),
            last_new_ip: None,
            status_message: None,
            target_active: false,
            address_display_mode: AddressDisplayMode::Masked,
        }
    }

    pub fn restore_data(
        &mut self,
        ipv4_masked: &[String],
        ipv4_raw: &[String],
        ipv6_masked: &[String],
        ipv6_raw: &[String],
    ) {
        for subnet in ipv4_masked {
            if self.seen_canonical_addresses.insert(subnet.clone()) {
                self.ipv4_masked_subnets.push(subnet.clone());
            }
        }

        for address in ipv4_raw {
            if self.seen_ipv4_raw.insert(address.clone()) {
                self.ipv4_raw_addresses.push(address.clone());
            }
        }

        for subnet in ipv6_masked {
            if self.seen_canonical_addresses.insert(subnet.clone()) {
                self.ipv6_masked_addresses.push(subnet.clone());
            }
        }

        for address in ipv6_raw {
            if self.seen_ipv6_raw.insert(address.clone()) {
                self.ipv6_raw_addresses.push(address.clone());
            }
        }
    }

    pub fn ipv4_masked_data(&self) -> &[String] {
        &self.ipv4_masked_subnets
    }

    pub fn ipv4_raw_data(&self) -> &[String] {
        &self.ipv4_raw_addresses
    }

    pub fn ipv6_masked_data(&self) -> &[String] {
        &self.ipv6_masked_addresses
    }

    pub fn ipv6_raw_data(&self) -> &[String] {
        &self.ipv6_raw_addresses
    }

    fn handle_event(&mut self, event: MonitorEvent) {
        match event {
            MonitorEvent::NewAddress(addr) => {
                let addr_str = addr.to_string();
                if !self.seen_canonical_addresses.insert(addr_str.clone()) {
                    return;
                }

                self.last_new_ip = Some(Instant::now());
                match addr {
                    DiscoveredAddress::Ipv4Subnet(_) => {
                        self.ipv4_masked_subnets.push(addr_str);
                    }
                    DiscoveredAddress::Ipv6Subnet64(_) => {
                        self.ipv6_masked_addresses.push(addr_str);
                    }
                }
            }
            MonitorEvent::NewIpv4Raw(raw_addr) => {
                let raw_str = raw_addr.to_string();
                if self.seen_ipv4_raw.insert(raw_str.clone()) {
                    self.ipv4_raw_addresses.push(raw_str);
                }
            }
            MonitorEvent::NewIpv6Raw(raw_addr) => {
                let raw_str = raw_addr.to_string();
                if self.seen_ipv6_raw.insert(raw_str.clone()) {
                    self.ipv6_raw_addresses.push(raw_str);
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

    fn toggle_address_display_mode(&mut self) {
        self.address_display_mode = self.address_display_mode.toggle();
        self.status_message = Some((
            format!("Address view: {}", self.address_display_mode.label()),
            Instant::now(),
        ));
    }

    fn export_to_file(&mut self) -> Result<String> {
        let filename = format!("net-use-export-{}.txt", chrono_timestamp());
        let mut file = std::fs::File::create(&filename)?;
        for subnet in &self.ipv4_masked_subnets {
            writeln!(file, "{subnet}")?;
        }
        for subnet in &self.ipv6_masked_addresses {
            writeln!(file, "{subnet}")?;
        }
        Ok(filename)
    }

    fn copy_to_clipboard(&self) -> Result<()> {
        let mut content = String::new();
        for subnet in &self.ipv4_masked_subnets {
            content.push_str(subnet);
            content.push('\n');
        }
        for subnet in &self.ipv6_masked_addresses {
            content.push_str(subnet);
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
            .map(|proc_info| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("PID {:<6}", proc_info.pid),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(&proc_info.name, Style::default().fg(Color::White)),
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

    fn current_ipv4_entries(&self) -> &[String] {
        match self.address_display_mode {
            AddressDisplayMode::Masked => &self.ipv4_masked_subnets,
            AddressDisplayMode::Raw => &self.ipv4_raw_addresses,
        }
    }

    fn current_ipv6_entries(&self) -> &[String] {
        match self.address_display_mode {
            AddressDisplayMode::Masked => &self.ipv6_masked_addresses,
            AddressDisplayMode::Raw => &self.ipv6_raw_addresses,
        }
    }

    fn render_ipv4(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let entries = self.current_ipv4_entries();
        let items: Vec<ListItem> = entries
            .iter()
            .map(|address| {
                ListItem::new(Span::styled(
                    address.as_str(),
                    Style::default().fg(Color::White),
                ))
            })
            .collect();

        let title = match self.address_display_mode {
            AddressDisplayMode::Masked => format!(" IPv4 Subnets [Masked] ({}) ", entries.len()),
            AddressDisplayMode::Raw => format!(" IPv4 Addresses [Raw] ({}) ", entries.len()),
        };
        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(list, area);
    }

    fn render_ipv6(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let entries = self.current_ipv6_entries();
        let items: Vec<ListItem> = entries
            .iter()
            .map(|address| {
                ListItem::new(Span::styled(
                    address.as_str(),
                    Style::default().fg(Color::White),
                ))
            })
            .collect();

        let title = match self.address_display_mode {
            AddressDisplayMode::Masked => format!(" IPv6 Subnets [Masked] ({}) ", entries.len()),
            AddressDisplayMode::Raw => format!(" IPv6 Addresses [Raw] ({}) ", entries.len()),
        };
        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(list, area);
    }

    fn current_total(&self) -> usize {
        self.current_ipv4_entries().len() + self.current_ipv6_entries().len()
    }

    fn render_status_bar(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let total = self.current_total();

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
            .filter(|(_, ts)| ts.elapsed() < Duration::from_secs(5))
            .map(|(msg, _)| format!("  {msg}"))
            .unwrap_or_default();

        let line1 = Line::from(vec![
            Span::styled(
                format!(" Total: {total} "),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!(
                    "│ {stability} │ Address view: {}",
                    self.address_display_mode.label()
                ),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(status_msg, Style::default().fg(Color::Green)),
        ]);
        let line2 = Line::from(vec![Span::styled(
            " [S]witch Mask/Raw  [E]xport(masked)  [C]opy(masked)  [Esc]Back  [Q]uit",
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

            terminal.draw(|frame| self.render(frame))?;

            if let Some(evt) = poll_event(Duration::from_millis(50))? {
                match evt {
                    AppEvent::Key(key) => {
                        if is_quit(&key) {
                            return Ok(MonitorAction::Quit);
                        }
                        match key.code {
                            KeyCode::Esc => return Ok(MonitorAction::Back),
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                self.toggle_address_display_mode();
                            }
                            KeyCode::Char('e') | KeyCode::Char('E') => {
                                match self.export_to_file() {
                                    Ok(filename) => {
                                        self.status_message = Some((
                                            format!("Exported to {filename}"),
                                            Instant::now(),
                                        ));
                                    }
                                    Err(err) => {
                                        self.status_message =
                                            Some((format!("Export failed: {err}"), Instant::now()));
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
                                    Err(err) => {
                                        self.status_message =
                                            Some((format!("Copy failed: {err}"), Instant::now()));
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

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use crate::types::{DiscoveredAddress, MonitorEvent};

    use super::{AddressDisplayMode, AppInfo, MonitorView};

    fn test_app_info() -> AppInfo {
        AppInfo {
            display_name: "Test App".to_string(),
            bundle_id: Some("com.example.test".to_string()),
            executable_name: "test".to_string(),
            app_path: None,
            pid: None,
        }
    }

    #[test]
    fn test_toggle_switches_ipv4_and_ipv6_together() {
        let mut view = MonitorView::new(test_app_info());

        view.handle_event(MonitorEvent::NewAddress(DiscoveredAddress::from_ipv4(
            "142.250.80.37".parse::<Ipv4Addr>().unwrap(),
        )));
        view.handle_event(MonitorEvent::NewAddress(DiscoveredAddress::from_ipv6(
            "2607:6bc0::10".parse::<Ipv6Addr>().unwrap(),
        )));

        view.handle_event(MonitorEvent::NewIpv4Raw(
            "142.250.80.37".parse::<Ipv4Addr>().unwrap(),
        ));
        view.handle_event(MonitorEvent::NewIpv4Raw(
            "142.250.80.99".parse::<Ipv4Addr>().unwrap(),
        ));
        view.handle_event(MonitorEvent::NewIpv6Raw(
            "2607:6bc0::10".parse::<Ipv6Addr>().unwrap(),
        ));
        view.handle_event(MonitorEvent::NewIpv6Raw(
            "2607:6bc0::11".parse::<Ipv6Addr>().unwrap(),
        ));

        assert_eq!(view.address_display_mode, AddressDisplayMode::Masked);
        assert_eq!(view.current_ipv4_entries(), &["142.250.80.0/24"]);
        assert_eq!(view.current_ipv6_entries(), &["2607:6bc0::/64"]);

        view.toggle_address_display_mode();

        assert_eq!(view.address_display_mode, AddressDisplayMode::Raw);
        assert_eq!(view.current_ipv4_entries().len(), 2);
        assert_eq!(view.current_ipv6_entries().len(), 2);
    }

    #[test]
    fn test_restore_data_keeps_masked_and_raw_sets() {
        let mut view = MonitorView::new(test_app_info());

        let ipv4_masked = vec!["142.250.80.0/24".to_string()];
        let ipv4_raw = vec!["142.250.80.37".to_string(), "142.250.80.99".to_string()];
        let ipv6_masked = vec!["2607:6bc0::/64".to_string()];
        let ipv6_raw = vec!["2607:6bc0::10".to_string(), "2607:6bc0::11".to_string()];

        view.restore_data(&ipv4_masked, &ipv4_raw, &ipv6_masked, &ipv6_raw);

        assert_eq!(view.ipv4_masked_data(), &["142.250.80.0/24"]);
        assert_eq!(view.ipv4_raw_data().len(), 2);
        assert_eq!(view.ipv6_masked_data(), &["2607:6bc0::/64"]);
        assert_eq!(view.ipv6_raw_data().len(), 2);
    }
}
