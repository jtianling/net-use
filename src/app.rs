use std::collections::HashSet;

use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

use crate::discovery::installed_apps;
use crate::monitor::aggregator::Aggregator;
use crate::monitor::connection;
use crate::monitor::process_tree;
use crate::types::{AppError, MonitorEvent, MonitorTarget};

pub struct MonitorEngine {
    target: MonitorTarget,
    tracked_pids: HashSet<i32>,
    aggregator: Aggregator,
    root_pids: Vec<i32>,
    app_path_prefix: Option<String>,
    executable_name: Option<String>,
}

impl MonitorEngine {
    pub fn new(target: MonitorTarget) -> Self {
        Self {
            target,
            tracked_pids: HashSet::new(),
            aggregator: Aggregator::new(),
            root_pids: Vec::new(),
            app_path_prefix: None,
            executable_name: None,
        }
    }

    pub fn resolve_target(&mut self) -> Result<bool, AppError> {
        match &self.target {
            MonitorTarget::Pid(pid) => {
                let pid = *pid;
                if process_tree::get_pid_name(pid).is_some() {
                    self.root_pids = vec![pid];
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            MonitorTarget::Name(name) => {
                let pids = process_tree::find_pids_by_name(name)?;
                if pids.is_empty() {
                    self.executable_name = Some(name.clone());
                    Ok(false)
                } else {
                    self.root_pids = pids;
                    Ok(true)
                }
            }
            MonitorTarget::Bundle(bundle_id) => {
                let (exec_name, app_path) = installed_apps::resolve_bundle_executable(bundle_id)
                    .ok_or_else(|| AppError::BundleNotFound(bundle_id.clone()))?;

                self.app_path_prefix = Some(app_path.clone());
                self.executable_name = Some(exec_name);

                let pids = process_tree::find_pids_by_executable_path_prefix(&app_path)?;
                if pids.is_empty() {
                    Ok(false)
                } else {
                    self.root_pids = pids;
                    Ok(true)
                }
            }
        }
    }

    fn refresh_process_tree(&mut self, tx: &mpsc::UnboundedSender<MonitorEvent>) {
        let mut new_tracked = HashSet::new();

        if let Some(prefix) = &self.app_path_prefix {
            if let Ok(pids) = process_tree::find_pids_by_executable_path_prefix(prefix) {
                self.root_pids = pids;
            }
        } else if let Some(name) = &self.executable_name
            && let Ok(pids) = process_tree::find_pids_by_name(name)
        {
            self.root_pids = pids;
        }

        for &root_pid in &self.root_pids {
            let descendants = process_tree::collect_descendants(root_pid);
            new_tracked.extend(descendants);
        }

        for &pid in &new_tracked {
            if !self.tracked_pids.contains(&pid)
                && let Some(info) = process_tree::get_process_info(pid)
            {
                let _ = tx.send(MonitorEvent::ProcessAdded(info));
            }
        }

        for &pid in &self.tracked_pids {
            if !new_tracked.contains(&pid) {
                let _ = tx.send(MonitorEvent::ProcessRemoved(pid));
            }
        }

        self.tracked_pids = new_tracked;
    }

    fn collect_connections(&mut self, tx: &mpsc::UnboundedSender<MonitorEvent>) {
        for &pid in &self.tracked_pids {
            let addrs = connection::collect_remote_addrs(pid);
            for addr in addrs {
                let result = self.aggregator.add(addr);
                if let Some(discovered) = result.discovered {
                    let _ = tx.send(MonitorEvent::NewAddress(discovered));
                }
                if let Some(raw_ipv4) = result.raw_ipv4 {
                    let _ = tx.send(MonitorEvent::NewIpv4Raw(raw_ipv4));
                }
                if let Some(raw_ipv6) = result.raw_ipv6 {
                    let _ = tx.send(MonitorEvent::NewIpv6Raw(raw_ipv6));
                }
            }
        }
    }
}

pub async fn run_monitor_loop(
    target: MonitorTarget,
    tx: mpsc::UnboundedSender<MonitorEvent>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    let mut engine = MonitorEngine::new(target);
    let mut target_found = false;

    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    return;
                }
            }
            _ = sleep(Duration::from_millis(200)) => {}
        }

        match engine.resolve_target() {
            Ok(true) => {
                if !target_found {
                    target_found = true;
                    let _ = tx.send(MonitorEvent::TargetFound);
                }
                engine.refresh_process_tree(&tx);
                engine.collect_connections(&tx);
            }
            Ok(false) => {
                if target_found {
                    target_found = false;
                    let _ = tx.send(MonitorEvent::TargetLost);
                }
            }
            Err(_) => {
                if target_found {
                    target_found = false;
                    let _ = tx.send(MonitorEvent::TargetLost);
                }
            }
        }
    }
}
