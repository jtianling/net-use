use crate::monitor::process_tree;
use crate::types::AppInfo;

pub fn discover_running_apps(installed: &[AppInfo]) -> Vec<AppInfo> {
    let all_pids = match process_tree::list_all_pids() {
        Ok(pids) => pids,
        Err(_) => return Vec::new(),
    };

    let mut running = Vec::new();

    for pid in &all_pids {
        let path = match process_tree::get_pid_path(*pid) {
            Some(p) => p,
            None => continue,
        };

        for app in installed {
            let app_path = match &app.app_path {
                Some(p) => p,
                None => continue,
            };

            if path.starts_with(app_path) {
                if !running
                    .iter()
                    .any(|a: &AppInfo| a.bundle_id == app.bundle_id && a.pid == Some(*pid))
                {
                    running.push(AppInfo {
                        display_name: app.display_name.clone(),
                        bundle_id: app.bundle_id.clone(),
                        executable_name: app.executable_name.clone(),
                        app_path: app.app_path.clone(),
                        pid: Some(*pid),
                    });
                }
                break;
            }
        }
    }

    running.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });
    running.dedup_by(|a, b| a.bundle_id == b.bundle_id);
    running
}

pub fn merge_app_lists(installed: Vec<AppInfo>, running: Vec<AppInfo>) -> Vec<AppInfo> {
    let mut merged = Vec::new();

    for r in &running {
        merged.push(r.clone());
    }

    for app in installed {
        let already_listed = running.iter().any(|r| r.bundle_id == app.bundle_id);
        if !already_listed {
            merged.push(app);
        }
    }

    merged
}
