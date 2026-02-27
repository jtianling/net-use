mod app;
mod discovery;
mod monitor;
mod tui;
mod types;

use std::io;

use anyhow::{Result, bail};
use clap::Parser;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use tokio::sync::{mpsc, watch};

use crate::discovery::installed_apps;
use crate::discovery::running_apps;
use crate::tui::app_selector::AppSelector;
use crate::tui::monitor_view::{MonitorAction, MonitorView};
use crate::types::{AppError, AppInfo, MonitorEvent, MonitorTarget};

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

    let mut preserved: Option<(
        MonitorTarget,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<String>,
    )> = None;

    loop {
        let (tx, mut rx) = mpsc::unbounded_channel::<MonitorEvent>();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let target_clone = target.clone();
        let monitor_handle = tokio::spawn(app::run_monitor_loop(target_clone, tx, shutdown_rx));

        let app_info = target_to_app_info(&target);
        let mut view = MonitorView::new(app_info);

        if let Some((ref prev_target, ref ipv4_masked, ref ipv4_raw, ref ipv6_masked, ref ipv6_raw)) =
            preserved
            && *prev_target == target
        {
            view.restore_data(ipv4_masked, ipv4_raw, ipv6_masked, ipv6_raw);
        }

        let action = view.run(terminal, &mut rx)?;

        preserved = Some((
            target.clone(),
            view.ipv4_masked_data().to_vec(),
            view.ipv4_raw_data().to_vec(),
            view.ipv6_masked_data().to_vec(),
            view.ipv6_raw_data().to_vec(),
        ));

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
