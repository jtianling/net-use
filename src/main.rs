mod app;
mod discovery;
mod monitor;
mod tui;
mod types;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use clap::Parser;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, watch};

use crate::discovery::installed_apps;
use crate::discovery::running_apps;
use crate::tui::app_selector::AppSelector;
use crate::tui::monitor_view::{MonitorAction, MonitorView};
use crate::types::{AppError, AppInfo, MonitorEvent, MonitorTarget};

const PRESERVED_HISTORY_FILE_NAME: &str = ".net-use-address-history.json";

#[derive(Parser, Debug)]
#[command(
    name = "net-use",
    about = "Monitor network connections of a specific app on macOS"
)]
pub struct Cli {
    #[arg(long, help = "Target process ID")]
    pub pid: Option<i32>,

    #[arg(long, help = "Target process name")]
    pub name: Option<String>,

    #[arg(long, help = "Target app Bundle ID (e.g., com.google.Chrome)")]
    pub bundle: Option<String>,

    #[arg(long, help = "Disable TUI, output to stdout")]
    pub no_tui: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PreservedData {
    ipv4_masked: Vec<String>,
    ipv4_raw: Vec<String>,
    ipv6_masked: Vec<String>,
    ipv6_raw: Vec<String>,
}

impl PreservedData {
    fn from_view(view: &MonitorView) -> Self {
        Self {
            ipv4_masked: view.ipv4_masked_data().to_vec(),
            ipv4_raw: view.ipv4_raw_data().to_vec(),
            ipv6_masked: view.ipv6_masked_data().to_vec(),
            ipv6_raw: view.ipv6_raw_data().to_vec(),
        }
    }

    fn restore_into(&self, view: &mut MonitorView) {
        view.restore_data(
            &self.ipv4_masked,
            &self.ipv4_raw,
            &self.ipv6_masked,
            &self.ipv6_raw,
        );
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PreservedStore {
    entries: Vec<PreservedStoreEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PreservedStoreEntry {
    target: MonitorTarget,
    data: PreservedData,
}

fn preserved_history_path_from(base_dir: &Path) -> PathBuf {
    base_dir.join(PRESERVED_HISTORY_FILE_NAME)
}

fn load_preserved_history() -> HashMap<MonitorTarget, PreservedData> {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Failed to resolve current directory for preserved history: {err}");
            return HashMap::new();
        }
    };

    let path = preserved_history_path_from(&cwd);
    load_preserved_history_from(&path)
}

fn load_preserved_history_from(path: &Path) -> HashMap<MonitorTarget, PreservedData> {
    let raw = match fs::read(path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return HashMap::new(),
        Err(err) => {
            eprintln!(
                "Failed to read preserved history from {}: {err}",
                path.display()
            );
            return HashMap::new();
        }
    };

    let store: PreservedStore = match serde_json::from_slice(&raw) {
        Ok(decoded) => decoded,
        Err(err) => {
            eprintln!(
                "Failed to parse preserved history from {}: {err}",
                path.display()
            );
            return HashMap::new();
        }
    };

    store
        .entries
        .into_iter()
        .map(|entry| (entry.target, entry.data))
        .collect()
}

fn persist_preserved_history(cache: &HashMap<MonitorTarget, PreservedData>) {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Failed to resolve current directory for preserved history: {err}");
            return;
        }
    };

    let path = preserved_history_path_from(&cwd);
    if let Err(err) = persist_preserved_history_to(&path, cache) {
        eprintln!("Failed to persist history to {}: {err}", path.display());
    }
}

fn persist_preserved_history_to(
    path: &Path,
    cache: &HashMap<MonitorTarget, PreservedData>,
) -> Result<()> {
    let store = PreservedStore {
        entries: cache
            .iter()
            .map(|(target, data)| PreservedStoreEntry {
                target: target.clone(),
                data: data.clone(),
            })
            .collect(),
    };

    let encoded = serde_json::to_vec_pretty(&store)?;
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, encoded)?;
    fs::rename(&temp_path, path)?;
    Ok(())
}

fn check_root() {
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("net-use requires root privileges to read process socket information.");
        eprintln!("Please run with: sudo net-use");
        std::process::exit(1);
    }
}

fn cli_to_target(cli: &Cli) -> Option<MonitorTarget> {
    if let Some(pid) = cli.pid {
        Some(MonitorTarget::Pid(pid))
    } else if let Some(ref name) = cli.name {
        Some(MonitorTarget::Name(name.clone()))
    } else {
        cli.bundle
            .as_ref()
            .map(|bundle| MonitorTarget::Bundle(bundle.clone()))
    }
}

fn validate_target(target: &MonitorTarget) -> Result<()> {
    if let MonitorTarget::Pid(pid) = target
        && monitor::process_tree::get_pid_name(*pid).is_none()
    {
        return Err(AppError::ProcessNotFound(*pid).into());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    check_root();

    if cli.no_tui {
        let target = match cli_to_target(&cli) {
            Some(t) => t,
            None => {
                bail!("--no-tui requires a target: --pid, --name, or --bundle");
            }
        };
        validate_target(&target)?;
        run_cli_mode(target).await
    } else {
        let target = cli_to_target(&cli);
        if let Some(ref t) = target {
            validate_target(t)?;
        }
        run_tui_mode(target).await
    }
}

async fn run_cli_mode(target: MonitorTarget) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<MonitorEvent>();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let monitor_handle = tokio::spawn(app::run_monitor_loop(target, tx, shutdown_rx));

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                match event {
                    MonitorEvent::NewAddress(addr) => {
                        println!("{addr}");
                    }
                    MonitorEvent::NewIpv4Raw(_) => {}
                    MonitorEvent::NewIpv6Raw(_) => {}
                    MonitorEvent::ProcessAdded(_)
                    | MonitorEvent::ProcessRemoved(_)
                    | MonitorEvent::TargetLost
                    | MonitorEvent::TargetFound => {}
                }
            }
            _ = &mut ctrl_c => {
                let _ = shutdown_tx.send(true);
                break;
            }
        }
    }

    let _ = monitor_handle.await;
    Ok(())
}

async fn run_tui_mode(initial_target: Option<MonitorTarget>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = tui_main_loop(&mut terminal, initial_target).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn tui_main_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    initial_target: Option<MonitorTarget>,
) -> Result<()> {
    let mut target = match initial_target {
        Some(t) => t,
        None => select_app(terminal)?,
    };

    let mut preserved_by_target: HashMap<MonitorTarget, PreservedData> = load_preserved_history();

    loop {
        let (tx, mut rx) = mpsc::unbounded_channel::<MonitorEvent>();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let target_clone = target.clone();
        let monitor_handle = tokio::spawn(app::run_monitor_loop(target_clone, tx, shutdown_rx));

        let app_info = target_to_app_info(&target);
        let mut view = MonitorView::new(app_info);

        if let Some(saved) = preserved_by_target.get(&target) {
            saved.restore_into(&mut view);
        }

        let action = view.run(terminal, &mut rx)?;

        preserved_by_target.insert(target.clone(), PreservedData::from_view(&view));
        persist_preserved_history(&preserved_by_target);

        let _ = shutdown_tx.send(true);
        let _ = monitor_handle.await;

        match action {
            MonitorAction::Quit => return Ok(()),
            MonitorAction::Back => {
                target = select_app(terminal)?;
            }
        }
    }
}

fn select_app(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> Result<MonitorTarget> {
    let installed = installed_apps::discover_installed_apps();
    let running = running_apps::discover_running_apps(&installed);
    let merged = running_apps::merge_app_lists(installed, running);
    let mut selector = AppSelector::new(merged);
    match selector.run(terminal)? {
        Some(app) => Ok(app_to_target(&app)),
        None => bail!("No app selected"),
    }
}

fn app_to_target(app: &AppInfo) -> MonitorTarget {
    if let Some(ref bundle_id) = app.bundle_id {
        MonitorTarget::Bundle(bundle_id.clone())
    } else if let Some(pid) = app.pid {
        MonitorTarget::Pid(pid)
    } else {
        MonitorTarget::Name(app.executable_name.clone())
    }
}

fn target_to_app_info(target: &MonitorTarget) -> AppInfo {
    match target {
        MonitorTarget::Pid(pid) => {
            let name =
                monitor::process_tree::get_pid_name(*pid).unwrap_or_else(|| format!("PID {pid}"));
            AppInfo {
                display_name: name,
                bundle_id: None,
                executable_name: String::new(),
                app_path: None,
                pid: Some(*pid),
            }
        }
        MonitorTarget::Name(name) => AppInfo {
            display_name: name.clone(),
            bundle_id: None,
            executable_name: name.clone(),
            app_path: None,
            pid: None,
        },
        MonitorTarget::Bundle(bundle_id) => {
            let display_name = installed_apps::resolve_bundle_executable(bundle_id)
                .map(|(name, _)| name)
                .unwrap_or_else(|| bundle_id.clone());
            AppInfo {
                display_name,
                bundle_id: Some(bundle_id.clone()),
                executable_name: String::new(),
                app_path: None,
                pid: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        PreservedData, load_preserved_history_from, persist_preserved_history_to,
        target_to_app_info,
    };
    use crate::tui::monitor_view::MonitorView;
    use crate::types::MonitorTarget;

    fn seeded_view(target: &MonitorTarget, ipv4_octet: u8, ipv6_segment: u16) -> MonitorView {
        let mut view = MonitorView::new(target_to_app_info(target));
        let ipv4_masked = vec![format!("10.0.{ipv4_octet}.0/24")];
        let ipv4_raw = vec![
            format!("10.0.{ipv4_octet}.1"),
            format!("10.0.{ipv4_octet}.2"),
        ];
        let ipv6_masked = vec![format!("2001:db8:{ipv6_segment:x}::/64")];
        let ipv6_raw = vec![
            format!("2001:db8:{ipv6_segment:x}::1"),
            format!("2001:db8:{ipv6_segment:x}::2"),
        ];
        view.restore_data(&ipv4_masked, &ipv4_raw, &ipv6_masked, &ipv6_raw);
        view
    }

    #[test]
    fn preserved_data_round_trip_restores_all_lists() {
        let target = MonitorTarget::Name("alpha".to_string());
        let view = seeded_view(&target, 0, 1);
        let snapshot = PreservedData::from_view(&view);

        let mut restored = MonitorView::new(target_to_app_info(&target));
        snapshot.restore_into(&mut restored);

        assert_eq!(restored.ipv4_masked_data(), &["10.0.0.0/24"]);
        assert_eq!(restored.ipv4_raw_data(), &["10.0.0.1", "10.0.0.2"]);
        assert_eq!(restored.ipv6_masked_data(), &["2001:db8:1::/64"]);
        assert_eq!(
            restored.ipv6_raw_data(),
            &["2001:db8:1::1", "2001:db8:1::2"]
        );
    }

    #[test]
    fn cache_isolated_per_target_and_keyed_by_monitor_target() {
        let target_a = MonitorTarget::Name("alpha".to_string());
        let target_b = MonitorTarget::Name("beta".to_string());

        let mut cache: HashMap<MonitorTarget, PreservedData> = HashMap::new();
        cache.insert(
            target_a.clone(),
            PreservedData::from_view(&seeded_view(&target_a, 0, 1)),
        );
        cache.insert(
            target_b.clone(),
            PreservedData::from_view(&seeded_view(&target_b, 1, 2)),
        );

        let mut restored_a = MonitorView::new(target_to_app_info(&MonitorTarget::Name(
            "alpha".to_string(),
        )));
        cache.get(&target_a).unwrap().restore_into(&mut restored_a);

        assert_eq!(restored_a.ipv4_masked_data(), &["10.0.0.0/24"]);
        assert!(!restored_a.ipv4_raw_data().contains(&"10.0.1.1".to_string()));

        let mut restored_b =
            MonitorView::new(target_to_app_info(&MonitorTarget::Name("beta".to_string())));
        cache.get(&target_b).unwrap().restore_into(&mut restored_b);

        assert_eq!(restored_b.ipv4_masked_data(), &["10.0.1.0/24"]);
        assert!(!restored_b.ipv4_raw_data().contains(&"10.0.0.1".to_string()));
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("net-use-{label}-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn preserved_history_file_round_trip_restores_cached_targets() {
        let target_a = MonitorTarget::Name("alpha".to_string());
        let target_b = MonitorTarget::Bundle("com.example.beta".to_string());

        let mut cache: HashMap<MonitorTarget, PreservedData> = HashMap::new();
        cache.insert(
            target_a.clone(),
            PreservedData::from_view(&seeded_view(&target_a, 9, 9)),
        );
        cache.insert(
            target_b.clone(),
            PreservedData::from_view(&seeded_view(&target_b, 10, 10)),
        );

        let dir = unique_temp_dir("preserved-history");
        let history_path = dir.join(".net-use-address-history.json");

        persist_preserved_history_to(&history_path, &cache).unwrap();
        let loaded = load_preserved_history_from(&history_path);

        let alpha = loaded.get(&target_a).unwrap();
        assert_eq!(alpha.ipv4_masked, vec!["10.0.9.0/24".to_string()]);
        assert_eq!(alpha.ipv6_masked, vec!["2001:db8:9::/64".to_string()]);

        let beta = loaded.get(&target_b).unwrap();
        assert_eq!(beta.ipv4_raw, vec!["10.0.10.1", "10.0.10.2"]);
        assert_eq!(beta.ipv6_raw, vec!["2001:db8:a::1", "2001:db8:a::2"]);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_preserved_history_missing_file_returns_empty_cache() {
        let dir = unique_temp_dir("missing-history");
        let history_path = dir.join(".net-use-address-history.json");

        let loaded = load_preserved_history_from(&history_path);
        assert!(loaded.is_empty());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_preserved_history_invalid_json_returns_empty_cache() {
        let dir = unique_temp_dir("invalid-history");
        let history_path = dir.join(".net-use-address-history.json");
        fs::write(&history_path, b"{not-valid-json").unwrap();

        let loaded = load_preserved_history_from(&history_path);
        assert!(loaded.is_empty());

        fs::remove_dir_all(dir).unwrap();
    }
}
