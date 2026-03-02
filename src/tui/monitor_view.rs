use std::collections::HashSet;
use std::io::{self, Write as _};
use std::net::Ipv4Addr;
use std::process::Command;
use std::time::{Duration, Instant};
use std::{cmp::Ordering, str::FromStr};

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

const MAX_COMMAND_SUMMARY_CHARS: usize = 48;
const MAX_PROCESS_ROWS: usize = 4;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddressOrderMode {
    Time,
    Alphabetical,
}

impl AddressOrderMode {
    fn label(self) -> &'static str {
        match self {
            Self::Time => "Time",
            Self::Alphabetical => "Sorted",
        }
    }

    fn toggle(self) -> Self {
        match self {
            Self::Time => Self::Alphabetical,
            Self::Alphabetical => Self::Time,
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
    paused: bool,
    address_display_mode: AddressDisplayMode,
    address_order_mode: AddressOrderMode,
    ipv4_scroll_offset: usize,
    ipv4_visible_rows: usize,
    ipv6_scroll_offset: usize,
    ipv6_visible_rows: usize,
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
            paused: false,
            address_display_mode: AddressDisplayMode::Masked,
            address_order_mode: AddressOrderMode::Time,
            ipv4_scroll_offset: 0,
            ipv4_visible_rows: 1,
            ipv6_scroll_offset: 0,
            ipv6_visible_rows: 1,
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

    pub fn set_paused(&mut self, paused: bool) {
        if paused {
            self.target_active = false;
            self.processes.clear();
            let message = if self.paused {
                "Monitoring already paused"
            } else {
                "Monitoring paused"
            };
            self.status_message = Some((message.to_string(), Instant::now()));
        } else if self.paused {
            self.status_message = Some(("Monitoring resumed".to_string(), Instant::now()));
        }
        self.paused = paused;
    }

    #[cfg(test)]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn drain_events(&mut self, rx: &mut mpsc::UnboundedReceiver<MonitorEvent>) {
        while let Ok(event) = rx.try_recv() {
            self.handle_event(event);
        }
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
                self.paused = false;
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
        self.clamp_ipv4_scroll_offset(self.current_ipv4_entries().len());
        self.clamp_ipv6_scroll_offset(self.current_ipv6_entries().len());
        self.status_message = Some((
            format!("Address view: {}", self.address_display_mode.label()),
            Instant::now(),
        ));
    }

    fn toggle_address_order_mode(&mut self) {
        self.address_order_mode = self.address_order_mode.toggle();
        self.status_message = Some((
            format!("Address order: {}", self.address_order_mode.label()),
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

        let status_indicator = if self.paused {
            Span::styled("◼ Paused", Style::default().fg(Color::Red))
        } else if self.target_active {
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
            .take(MAX_PROCESS_ROWS)
            .map(|proc_info| {
                ListItem::new(Span::styled(
                    format_process_summary(proc_info),
                    Style::default().fg(Color::White),
                ))
            })
            .collect();

        let remaining = if self.processes.len() > MAX_PROCESS_ROWS {
            format!(
                " Tracked Processes (+{} more) ",
                self.processes.len() - MAX_PROCESS_ROWS
            )
        } else {
            " Tracked Processes ".to_string()
        };

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(remaining));
        frame.render_widget(list, area);
    }

    fn parse_ipv4_sort_key(entry: &str) -> Option<[u8; 4]> {
        let (address, _) = entry.split_once('/').unwrap_or((entry, ""));
        Ipv4Addr::from_str(address).ok().map(|ip| ip.octets())
    }

    fn ordered_ipv4_entries<'a>(&self, entries: &'a [String]) -> Vec<&'a str> {
        let mut ordered = entries.iter().map(String::as_str).collect::<Vec<_>>();
        if self.address_order_mode == AddressOrderMode::Alphabetical {
            ordered.sort_by(|left, right| {
                match (
                    Self::parse_ipv4_sort_key(left),
                    Self::parse_ipv4_sort_key(right),
                ) {
                    (Some(left_key), Some(right_key)) => left_key.cmp(&right_key),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => left.cmp(right),
                }
            });
        }
        ordered
    }

    fn ordered_ipv6_entries<'a>(&self, entries: &'a [String]) -> Vec<&'a str> {
        let mut ordered = entries.iter().map(String::as_str).collect::<Vec<_>>();
        if self.address_order_mode == AddressOrderMode::Alphabetical {
            ordered.sort_unstable();
        }
        ordered
    }

    fn current_ipv4_entries(&self) -> Vec<&str> {
        let entries = match self.address_display_mode {
            AddressDisplayMode::Masked => &self.ipv4_masked_subnets,
            AddressDisplayMode::Raw => &self.ipv4_raw_addresses,
        };
        self.ordered_ipv4_entries(entries)
    }

    fn current_ipv6_entries(&self) -> Vec<&str> {
        let entries = match self.address_display_mode {
            AddressDisplayMode::Masked => &self.ipv6_masked_addresses,
            AddressDisplayMode::Raw => &self.ipv6_raw_addresses,
        };
        self.ordered_ipv6_entries(entries)
    }

    fn max_scroll_offset(total_rows: usize, visible_rows: usize) -> usize {
        total_rows.saturating_sub(visible_rows.max(1))
    }

    fn clamp_ipv4_scroll_offset(&mut self, total_rows: usize) {
        let max_offset = Self::max_scroll_offset(total_rows, self.ipv4_visible_rows);
        self.ipv4_scroll_offset = self.ipv4_scroll_offset.min(max_offset);
    }

    fn clamp_ipv6_scroll_offset(&mut self, total_rows: usize) {
        let max_offset = Self::max_scroll_offset(total_rows, self.ipv6_visible_rows);
        self.ipv6_scroll_offset = self.ipv6_scroll_offset.min(max_offset);
    }

    fn scroll_ipv4_up(&mut self, lines: usize) {
        self.ipv4_scroll_offset = self.ipv4_scroll_offset.saturating_sub(lines);
    }

    fn scroll_ipv4_down(&mut self, lines: usize, total_rows: usize) {
        let max_offset = Self::max_scroll_offset(total_rows, self.ipv4_visible_rows);
        self.ipv4_scroll_offset = self
            .ipv4_scroll_offset
            .saturating_add(lines)
            .min(max_offset);
    }

    fn scroll_ipv6_up(&mut self, lines: usize) {
        self.ipv6_scroll_offset = self.ipv6_scroll_offset.saturating_sub(lines);
    }

    fn scroll_ipv6_down(&mut self, lines: usize, total_rows: usize) {
        let max_offset = Self::max_scroll_offset(total_rows, self.ipv6_visible_rows);
        self.ipv6_scroll_offset = self
            .ipv6_scroll_offset
            .saturating_add(lines)
            .min(max_offset);
    }

    fn handle_scroll_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up => {
                self.scroll_ipv4_up(1);
                true
            }
            KeyCode::Down => {
                let total_rows = self.current_ipv4_entries().len();
                self.scroll_ipv4_down(1, total_rows);
                true
            }
            KeyCode::Char('k') | KeyCode::Char('K') => {
                self.scroll_ipv6_up(1);
                true
            }
            KeyCode::Char('j') | KeyCode::Char('J') => {
                let total_rows = self.current_ipv6_entries().len();
                self.scroll_ipv6_down(1, total_rows);
                true
            }
            _ => false,
        }
    }

    fn render_ipv4(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let total_rows = self.current_ipv4_entries().len();
        self.ipv4_visible_rows = usize::from(area.height.saturating_sub(2)).max(1);
        self.clamp_ipv4_scroll_offset(total_rows);

        let start = self.ipv4_scroll_offset.min(total_rows);
        let end = start.saturating_add(self.ipv4_visible_rows).min(total_rows);
        let entries = self.current_ipv4_entries();
        let visible_entries = entries.get(start..end).unwrap_or_default();
        let items: Vec<ListItem> = visible_entries
            .iter()
            .map(|entry| ListItem::new(Span::styled(*entry, Style::default().fg(Color::White))))
            .collect();

        let title_prefix = match self.address_display_mode {
            AddressDisplayMode::Masked => " IPv4 Subnets [Masked] ",
            AddressDisplayMode::Raw => " IPv4 Addresses [Raw] ",
        };
        let title = if total_rows > self.ipv4_visible_rows {
            let up = if start > 0 { "↑" } else { " " };
            let down = if end < total_rows { "↓" } else { " " };
            format!(
                "{title_prefix}({total_rows}) {up}{down} {}-{} / {} ",
                start + 1,
                end.max(start + 1),
                total_rows
            )
        } else {
            format!("{title_prefix}({total_rows}) ")
        };

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(list, area);
    }

    fn render_ipv6(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let total_rows = self.current_ipv6_entries().len();
        self.ipv6_visible_rows = usize::from(area.height.saturating_sub(2)).max(1);
        self.clamp_ipv6_scroll_offset(total_rows);

        let start = self.ipv6_scroll_offset.min(total_rows);
        let end = start.saturating_add(self.ipv6_visible_rows).min(total_rows);
        let entries = self.current_ipv6_entries();
        let visible_entries = entries.get(start..end).unwrap_or_default();
        let items: Vec<ListItem> = visible_entries
            .iter()
            .map(|entry| ListItem::new(Span::styled(*entry, Style::default().fg(Color::White))))
            .collect();

        let title_prefix = match self.address_display_mode {
            AddressDisplayMode::Masked => " IPv6 Subnets [Masked] ",
            AddressDisplayMode::Raw => " IPv6 Addresses [Raw] ",
        };
        let title = if total_rows > self.ipv6_visible_rows {
            let up = if start > 0 { "↑" } else { " " };
            let down = if end < total_rows { "↓" } else { " " };
            format!(
                "{title_prefix}({total_rows}) {up}{down} {}-{} / {} ",
                start + 1,
                end.max(start + 1),
                total_rows
            )
        } else {
            format!("{title_prefix}({total_rows}) ")
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
                    "│ {stability} │ Address view: {} │ Order: {}",
                    self.address_display_mode.label(),
                    self.address_order_mode.label()
                ),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(status_msg, Style::default().fg(Color::Green)),
        ]);
        let pause_label = if self.paused { "[P]Resume" } else { "[P]ause" };
        let line2 = Line::from(vec![Span::styled(
            format!(
                " [Up/Down]Scroll IPv4  [J/K]Scroll IPv6  [S]witch  [O]rder  {pause_label}  [E]xport(masked)  [C]opy(masked)  [Esc]Back  [Q]uit"
            ),
            Style::default().fg(Color::DarkGray),
        )]);

        let paragraph = Paragraph::new(vec![line1, line2]);
        frame.render_widget(paragraph, area);
    }

    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
        mut rx: Option<&mut mpsc::UnboundedReceiver<MonitorEvent>>,
        mut on_tick: impl FnMut(),
    ) -> Result<MonitorAction> {
        loop {
            on_tick();
            if let Some(receiver) = rx.as_deref_mut() {
                self.drain_events(receiver);
            }

            terminal.draw(|frame| self.render(frame))?;

            if let Some(evt) = poll_event(Duration::from_millis(50))? {
                match evt {
                    AppEvent::Key(key) => {
                        if is_quit(&key) {
                            return Ok(MonitorAction::Quit);
                        }
                        if let Some(action) = self.handle_key_code(key.code) {
                            return Ok(action);
                        }
                    }
                    AppEvent::Tick => {}
                }
            }
        }
    }

    fn handle_key_code(&mut self, key: KeyCode) -> Option<MonitorAction> {
        match key {
            KeyCode::Esc => Some(MonitorAction::Back),
            KeyCode::Char('p') | KeyCode::Char('P') => {
                if self.paused {
                    Some(MonitorAction::ResumeTarget)
                } else {
                    Some(MonitorAction::PauseTarget)
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.toggle_address_display_mode();
                None
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                self.toggle_address_order_mode();
                self.clamp_ipv4_scroll_offset(self.current_ipv4_entries().len());
                self.clamp_ipv6_scroll_offset(self.current_ipv6_entries().len());
                None
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                match self.export_to_file() {
                    Ok(filename) => {
                        self.status_message =
                            Some((format!("Exported to {filename}"), Instant::now()));
                    }
                    Err(err) => {
                        self.status_message =
                            Some((format!("Export failed: {err}"), Instant::now()));
                    }
                }
                None
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                match self.copy_to_clipboard() {
                    Ok(()) => {
                        self.status_message =
                            Some(("Copied to clipboard".to_string(), Instant::now()));
                    }
                    Err(err) => {
                        self.status_message = Some((format!("Copy failed: {err}"), Instant::now()));
                    }
                }
                None
            }
            _ => {
                self.handle_scroll_key(key);
                None
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorAction {
    Quit,
    Back,
    PauseTarget,
    ResumeTarget,
}

fn chrono_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

fn format_process_summary(proc_info: &ProcessInfo) -> String {
    let head = format!("PID {:<6} {}", proc_info.pid, proc_info.name);
    match proc_info.command.as_ref() {
        Some(command) if !command.is_empty() => {
            let summary = summarize_command(command, MAX_COMMAND_SUMMARY_CHARS);
            format!("{head}  {summary}")
        }
        _ => head,
    }
}

fn summarize_command(command: &str, max_chars: usize) -> String {
    let normalized = command.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        return normalized;
    }

    let mut shortened = normalized.chars().take(max_chars).collect::<String>();
    shortened.push_str("...");
    shortened
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use crossterm::event::KeyCode;

    use crate::types::{AppMonitorState, DiscoveredAddress, MonitorEvent};

    use crate::types::ProcessInfo;

    use super::{
        AddressDisplayMode, AddressOrderMode, AppInfo, MonitorAction, MonitorView,
        format_process_summary,
    };

    fn test_app_info() -> AppInfo {
        AppInfo {
            display_name: "Test App".to_string(),
            bundle_id: Some("com.example.test".to_string()),
            executable_name: "test".to_string(),
            app_path: None,
            pid: None,
            monitor_state: AppMonitorState::Unmonitored,
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
        assert_eq!(view.address_order_mode, AddressOrderMode::Time);
        assert_eq!(view.current_ipv4_entries(), vec!["142.250.80.0/24"]);
        assert_eq!(view.current_ipv6_entries(), vec!["2607:6bc0::/64"]);

        view.toggle_address_display_mode();

        assert_eq!(view.address_display_mode, AddressDisplayMode::Raw);
        assert_eq!(view.current_ipv4_entries().len(), 2);
        assert_eq!(view.current_ipv6_entries().len(), 2);
    }

    fn fill_masked_rows(view: &mut MonitorView, count: usize) {
        let ipv4_masked = (0..count)
            .map(|i| format!("10.0.{i}.0/24"))
            .collect::<Vec<_>>();
        let ipv6_masked = (0..count)
            .map(|i| format!("2001:db8:{i:x}::/64"))
            .collect::<Vec<_>>();
        view.restore_data(&ipv4_masked, &[], &ipv6_masked, &[]);
    }

    #[test]
    fn test_separate_scroll_offsets_clamp_to_bounds() {
        let mut view = MonitorView::new(test_app_info());
        fill_masked_rows(&mut view, 8);

        view.ipv4_visible_rows = 3;
        view.ipv6_visible_rows = 5;
        view.ipv4_scroll_offset = 1_000;
        view.ipv6_scroll_offset = 1_000;

        view.clamp_ipv4_scroll_offset(view.current_ipv4_entries().len());
        view.clamp_ipv6_scroll_offset(view.current_ipv6_entries().len());

        let expected_ipv4 = MonitorView::max_scroll_offset(view.current_ipv4_entries().len(), 3);
        let expected_ipv6 = MonitorView::max_scroll_offset(view.current_ipv6_entries().len(), 5);
        assert_eq!(view.ipv4_scroll_offset, expected_ipv4);
        assert_eq!(view.ipv6_scroll_offset, expected_ipv6);
    }

    #[test]
    fn test_scroll_keys_affect_target_window_only() {
        let mut view = MonitorView::new(test_app_info());
        fill_masked_rows(&mut view, 8);
        view.ipv4_visible_rows = 3;
        view.ipv6_visible_rows = 2;

        view.handle_key_code(KeyCode::Down);
        assert_eq!(view.ipv4_scroll_offset, 1);
        assert_eq!(view.ipv6_scroll_offset, 0);

        view.handle_key_code(KeyCode::Down);
        assert_eq!(view.ipv4_scroll_offset, 2);
        assert_eq!(view.ipv6_scroll_offset, 0);

        view.handle_key_code(KeyCode::Char('j'));
        assert_eq!(view.ipv4_scroll_offset, 2);
        assert_eq!(view.ipv6_scroll_offset, 1);

        for _ in 0..32 {
            view.handle_key_code(KeyCode::Char('j'));
        }
        let ipv6_max = MonitorView::max_scroll_offset(view.current_ipv6_entries().len(), 2);
        assert_eq!(view.ipv6_scroll_offset, ipv6_max);

        view.handle_key_code(KeyCode::Char('k'));
        assert_eq!(view.ipv6_scroll_offset, ipv6_max.saturating_sub(1));
        assert_eq!(view.ipv4_scroll_offset, 2);

        view.handle_key_code(KeyCode::Up);
        assert_eq!(view.ipv4_scroll_offset, 1);
        assert_eq!(view.ipv6_scroll_offset, ipv6_max.saturating_sub(1));
    }

    #[test]
    fn test_pause_key_returns_pause_action() {
        let mut view = MonitorView::new(test_app_info());
        assert_eq!(
            view.handle_key_code(KeyCode::Char('p')),
            Some(MonitorAction::PauseTarget)
        );
        assert_eq!(
            view.handle_key_code(KeyCode::Char('P')),
            Some(MonitorAction::PauseTarget)
        );
    }

    #[test]
    fn test_pause_key_toggles_to_resume_when_paused() {
        let mut view = MonitorView::new(test_app_info());
        view.set_paused(true);
        assert_eq!(
            view.handle_key_code(KeyCode::Char('p')),
            Some(MonitorAction::ResumeTarget)
        );
        assert_eq!(
            view.handle_key_code(KeyCode::Char('P')),
            Some(MonitorAction::ResumeTarget)
        );
    }

    #[test]
    fn test_escape_returns_back_action() {
        let mut view = MonitorView::new(test_app_info());
        assert_eq!(
            view.handle_key_code(KeyCode::Esc),
            Some(MonitorAction::Back)
        );
    }

    #[test]
    fn test_set_paused_is_idempotent() {
        let mut view = MonitorView::new(test_app_info());
        view.set_paused(true);
        assert!(view.is_paused());
        view.set_paused(true);
        assert!(view.is_paused());
    }

    #[test]
    fn test_toggle_while_scrolled_clamps_each_window_offset() {
        let mut view = MonitorView::new(test_app_info());
        let ipv4_masked = vec!["142.250.80.0/24".to_string()];
        let ipv4_raw = (0..10)
            .map(|i| format!("142.250.80.{}", i + 1))
            .collect::<Vec<_>>();
        let ipv6_masked = vec!["2607:6bc0::/64".to_string()];
        let ipv6_raw = (0..10)
            .map(|i| format!("2607:6bc0::{:x}", i + 1))
            .collect::<Vec<_>>();
        view.restore_data(&ipv4_masked, &ipv4_raw, &ipv6_masked, &ipv6_raw);

        view.ipv4_visible_rows = 3;
        view.ipv6_visible_rows = 4;
        view.toggle_address_display_mode();
        for _ in 0..12 {
            view.handle_key_code(KeyCode::Down);
            view.handle_key_code(KeyCode::Char('j'));
        }
        assert_eq!(view.address_display_mode, AddressDisplayMode::Raw);
        assert!(view.ipv4_scroll_offset > 0);
        assert!(view.ipv6_scroll_offset > 0);

        view.toggle_address_display_mode();
        assert_eq!(view.address_display_mode, AddressDisplayMode::Masked);
        let ipv4_max = MonitorView::max_scroll_offset(view.current_ipv4_entries().len(), 3);
        let ipv6_max = MonitorView::max_scroll_offset(view.current_ipv6_entries().len(), 4);
        assert!(view.ipv4_scroll_offset <= ipv4_max);
        assert!(view.ipv6_scroll_offset <= ipv6_max);
        assert_eq!(view.current_ipv4_entries(), &["142.250.80.0/24"]);
        assert_eq!(view.current_ipv6_entries(), &["2607:6bc0::/64"]);
    }

    #[test]
    fn test_toggle_order_switches_ipv4_and_ipv6_together() {
        let mut view = MonitorView::new(test_app_info());

        let ipv4_masked = vec![
            "10.0.2.0/24".to_string(),
            "10.0.10.0/24".to_string(),
            "10.0.1.0/24".to_string(),
        ];
        let ipv4_raw = vec![];
        let ipv6_masked = vec![
            "2001:db8:c::/64".to_string(),
            "2001:db8:a::/64".to_string(),
            "2001:db8:b::/64".to_string(),
        ];
        let ipv6_raw = vec![];

        view.restore_data(&ipv4_masked, &ipv4_raw, &ipv6_masked, &ipv6_raw);

        assert_eq!(view.address_order_mode, AddressOrderMode::Time);
        assert_eq!(
            view.current_ipv4_entries(),
            vec!["10.0.2.0/24", "10.0.10.0/24", "10.0.1.0/24"]
        );
        assert_eq!(
            view.current_ipv6_entries(),
            vec!["2001:db8:c::/64", "2001:db8:a::/64", "2001:db8:b::/64"]
        );

        view.toggle_address_order_mode();

        assert_eq!(view.address_order_mode, AddressOrderMode::Alphabetical);
        assert_eq!(
            view.current_ipv4_entries(),
            vec!["10.0.1.0/24", "10.0.2.0/24", "10.0.10.0/24"]
        );
        assert_eq!(
            view.current_ipv6_entries(),
            vec!["2001:db8:a::/64", "2001:db8:b::/64", "2001:db8:c::/64"]
        );
    }

    #[test]
    fn test_sorted_order_applies_in_raw_view() {
        let mut view = MonitorView::new(test_app_info());

        let ipv4_masked = vec![];
        let ipv4_raw = vec![
            "1.1.1.2".to_string(),
            "1.1.1.10".to_string(),
            "1.1.1.1".to_string(),
        ];
        let ipv6_masked = vec![];
        let ipv6_raw = vec![
            "2001:db8:c::1".to_string(),
            "2001:db8:a::1".to_string(),
            "2001:db8:b::1".to_string(),
        ];

        view.restore_data(&ipv4_masked, &ipv4_raw, &ipv6_masked, &ipv6_raw);

        view.toggle_address_display_mode();
        view.toggle_address_order_mode();

        assert_eq!(view.address_display_mode, AddressDisplayMode::Raw);
        assert_eq!(view.address_order_mode, AddressOrderMode::Alphabetical);
        assert_eq!(
            view.current_ipv4_entries(),
            vec!["1.1.1.1", "1.1.1.2", "1.1.1.10"]
        );
        assert_eq!(
            view.current_ipv6_entries(),
            vec!["2001:db8:a::1", "2001:db8:b::1", "2001:db8:c::1"]
        );
    }

    #[test]
    fn test_ipv4_sorted_order_uses_numeric_segments() {
        let mut view = MonitorView::new(test_app_info());

        let ipv4_masked = vec![
            "100.0.0.0/24".to_string(),
            "9.0.0.0/24".to_string(),
            "10.0.0.0/24".to_string(),
        ];
        view.restore_data(&ipv4_masked, &[], &[], &[]);
        view.toggle_address_order_mode();

        assert_eq!(
            view.current_ipv4_entries(),
            vec!["9.0.0.0/24", "10.0.0.0/24", "100.0.0.0/24"]
        );
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

    #[test]
    fn test_format_process_summary_with_command() {
        let process = ProcessInfo {
            pid: 123,
            name: "python".to_string(),
            command: Some("python worker.py --queue high-priority".to_string()),
        };

        let summary = format_process_summary(&process);

        assert!(summary.contains("PID 123"));
        assert!(summary.contains("python"));
        assert!(summary.contains("worker.py"));
    }

    #[test]
    fn test_format_process_summary_without_command() {
        let process = ProcessInfo {
            pid: 456,
            name: "curl".to_string(),
            command: None,
        };

        let summary = format_process_summary(&process);

        assert_eq!(summary, "PID 456    curl");
    }
}
