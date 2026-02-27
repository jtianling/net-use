use std::fs;
use std::path::{Path, PathBuf};

use crate::types::AppInfo;

fn scan_app_dir(dir: &Path) -> Vec<AppInfo> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut apps = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("app") {
            continue;
        }
        if let Some(info) = parse_app_bundle(&path) {
            apps.push(info);
        }
    }
    apps
}

fn parse_app_bundle(app_path: &Path) -> Option<AppInfo> {
    let plist_path = app_path.join("Contents/Info.plist");
    let value = plist::Value::from_file(&plist_path).ok()?;
    let dict = value.as_dictionary()?;

    let bundle_id = dict
        .get("CFBundleIdentifier")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string());

    let display_name = dict
        .get("CFBundleDisplayName")
        .or_else(|| dict.get("CFBundleName"))
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            app_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });

    let executable_name = dict
        .get("CFBundleExecutable")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())?;

    Some(AppInfo {
        display_name,
        bundle_id,
        executable_name,
        app_path: Some(app_path.to_string_lossy().into_owned()),
        pid: None,
    })
}

pub fn discover_installed_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();

    apps.extend(scan_app_dir(Path::new("/Applications")));

    if let Some(home) = dirs_home() {
        let user_apps = home.join("Applications");
        apps.extend(scan_app_dir(&user_apps));
    }

    apps.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });
    apps
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub fn resolve_bundle_executable(bundle_id: &str) -> Option<(String, String)> {
    let apps = discover_installed_apps();
    apps.into_iter()
        .find(|app| app.bundle_id.as_deref() == Some(bundle_id))
        .map(|app| {
            let app_path = app.app_path.unwrap_or_default();
            (app.executable_name, app_path)
        })
}
