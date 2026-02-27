use std::collections::HashSet;

use crate::monitor::process_tree;
use crate::types::AppInfo;

#[derive(Debug, Clone)]
struct ProcessSnapshot {
    pid: i32,
    path: Option<String>,
    name: Option<String>,
}

pub fn discover_running_apps(installed: &[AppInfo]) -> Vec<AppInfo> {
    let all_pids = match process_tree::list_all_pids() {
        Ok(pids) => pids,
        Err(_) => return Vec::new(),
    };

    let snapshots: Vec<ProcessSnapshot> = all_pids
        .iter()
        .map(|pid| ProcessSnapshot {
            pid: *pid,
            path: process_tree::get_pid_path(*pid),
            name: process_tree::get_pid_name(*pid),
        })
        .collect();

    collect_running_apps(installed, &snapshots)
}

fn collect_running_apps(installed: &[AppInfo], snapshots: &[ProcessSnapshot]) -> Vec<AppInfo> {
    let mut running: Vec<AppInfo> = snapshots
        .iter()
        .filter_map(|snapshot| {
            let matched_gui_app = snapshot.path.as_ref().and_then(|path| {
                installed.iter().find(|app| {
                    app.app_path
                        .as_ref()
                        .is_some_and(|app_path| path.starts_with(app_path))
                })
            });

            if let Some(app) = matched_gui_app {
                return Some(AppInfo {
                    display_name: app.display_name.clone(),
                    bundle_id: app.bundle_id.clone(),
                    executable_name: app.executable_name.clone(),
                    app_path: app.app_path.clone(),
                    pid: Some(snapshot.pid),
                });
            }

            snapshot.name.as_ref().map(|name| AppInfo {
                display_name: name.clone(),
                bundle_id: None,
                executable_name: name.clone(),
                app_path: None,
                pid: Some(snapshot.pid),
            })
        })
        .collect();

    running.sort_by(|a, b| {
        a.bundle_id
            .is_none()
            .cmp(&b.bundle_id.is_none())
            .then_with(|| {
                a.display_name
                    .to_lowercase()
                    .cmp(&b.display_name.to_lowercase())
            })
            .then_with(|| a.pid.unwrap_or(i32::MAX).cmp(&b.pid.unwrap_or(i32::MAX)))
    });

    dedup_running_apps(running)
}

fn dedup_running_apps(running: Vec<AppInfo>) -> Vec<AppInfo> {
    let mut seen_bundles = HashSet::new();
    let mut seen_cli_pids = HashSet::new();

    running
        .into_iter()
        .filter(|app| match (&app.bundle_id, app.pid) {
            (Some(bundle_id), _) => seen_bundles.insert(bundle_id.clone()),
            (None, Some(pid)) => seen_cli_pids.insert(pid),
            (None, None) => true,
        })
        .collect()
}

pub fn merge_app_lists(installed: Vec<AppInfo>, running: Vec<AppInfo>) -> Vec<AppInfo> {
    let mut merged = running;

    for app in installed {
        let already_listed = app.bundle_id.as_ref().is_some_and(|bundle_id| {
            merged
                .iter()
                .any(|running_app| running_app.bundle_id.as_ref() == Some(bundle_id))
        });
        if !already_listed {
            merged.push(app);
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::{ProcessSnapshot, collect_running_apps, merge_app_lists};
    use crate::types::AppInfo;

    fn installed_app(name: &str, bundle: Option<&str>, app_path: Option<&str>) -> AppInfo {
        AppInfo {
            display_name: name.to_string(),
            bundle_id: bundle.map(ToOwned::to_owned),
            executable_name: name.to_string(),
            app_path: app_path.map(ToOwned::to_owned),
            pid: None,
        }
    }

    fn snapshot(pid: i32, path: Option<&str>, name: Option<&str>) -> ProcessSnapshot {
        ProcessSnapshot {
            pid,
            path: path.map(ToOwned::to_owned),
            name: name.map(ToOwned::to_owned),
        }
    }

    #[test]
    fn test_collect_running_apps_includes_cli_process() {
        let installed = vec![installed_app(
            "Google Chrome",
            Some("com.google.Chrome"),
            Some("/Applications/Google Chrome.app"),
        )];
        let snapshots = vec![
            snapshot(
                111,
                Some("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
                Some("Google Chrome"),
            ),
            snapshot(222, Some("/usr/bin/curl"), Some("curl")),
        ];

        let running = collect_running_apps(&installed, &snapshots);

        assert_eq!(running.len(), 2);
        assert!(running.iter().any(|app| {
            app.bundle_id.as_deref() == Some("com.google.Chrome") && app.pid == Some(111)
        }));
        assert!(running.iter().any(|app| app.bundle_id.is_none()
            && app.display_name == "curl"
            && app.pid == Some(222)));
    }

    #[test]
    fn test_collect_running_apps_keeps_same_name_cli_with_different_pid() {
        let snapshots = vec![
            snapshot(1001, None, Some("python")),
            snapshot(1002, None, Some("python")),
        ];

        let running = collect_running_apps(&[], &snapshots);

        assert_eq!(running.len(), 2);
        assert!(running.iter().any(|app| app.pid == Some(1001)));
        assert!(running.iter().any(|app| app.pid == Some(1002)));
    }

    #[test]
    fn test_collect_running_apps_dedups_gui_bundle() {
        let installed = vec![installed_app(
            "Chrome",
            Some("com.google.Chrome"),
            Some("/Applications/Chrome.app"),
        )];
        let snapshots = vec![
            snapshot(
                301,
                Some("/Applications/Chrome.app/Contents/MacOS/Chrome"),
                Some("Chrome"),
            ),
            snapshot(
                302,
                Some("/Applications/Chrome.app/Contents/MacOS/Chrome Helper"),
                Some("Chrome Helper"),
            ),
        ];

        let running = collect_running_apps(&installed, &snapshots);

        assert_eq!(running.len(), 1);
        assert_eq!(running[0].bundle_id.as_deref(), Some("com.google.Chrome"));
    }

    #[test]
    fn test_merge_app_lists_does_not_skip_installed_for_cli_entries() {
        let installed = vec![installed_app(
            "Safari",
            Some("com.apple.Safari"),
            Some("/Applications/Safari.app"),
        )];
        let running = vec![installed_app("curl", None, None)];

        let merged = merge_app_lists(installed, running);

        assert_eq!(merged.len(), 2);
        assert!(
            merged
                .iter()
                .any(|app| app.bundle_id.as_deref() == Some("com.apple.Safari"))
        );
        assert!(merged.iter().any(|app| app.display_name == "curl"));
    }
}
