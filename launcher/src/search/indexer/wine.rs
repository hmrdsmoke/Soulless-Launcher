// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/search/indexer/wine.rs
// Indexes Windows apps installed via Wine prefixes.

use super::{AppEntry, AppSource};
use nucleo_matcher::Utf32String;
use std::fs;
use std::path::Path;

pub fn index() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // Default Wine prefix
    let default_prefix = home.join(".wine");
    if default_prefix.exists() {
        scan_prefix(&default_prefix, &mut apps);
    }

    // Bottles prefixes (~/.var/app/com.usebottles.bottles/data/bottles/bottles/)
    let bottles_dir = home.join(".var/app/com.usebottles.bottles/data/bottles/bottles");
    if bottles_dir.exists()
        && let Ok(entries) = fs::read_dir(&bottles_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_prefix(&path, &mut apps);
                }
            }
        }

    // PlayOnLinux prefixes
    let pol_dir = home.join(".PlayOnLinux/wineprefix");
    if pol_dir.exists()
        && let Ok(entries) = fs::read_dir(&pol_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_prefix(&path, &mut apps);
                }
            }
        }

    apps
}

fn scan_prefix(prefix: &Path, apps: &mut Vec<AppEntry>) {
    // Look for .lnk shortcuts in Start Menu
    let start_menu = prefix.join("drive_c/users/Public/Start Menu/Programs");
    scan_shortcuts(&start_menu, apps);
    let start_menu2 = prefix.join("drive_c/ProgramData/Microsoft/Windows/Start Menu/Programs");
    scan_shortcuts(&start_menu2, apps);
}

fn scan_shortcuts(dir: &Path, apps: &mut Vec<AppEntry>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_shortcuts(&path, apps);
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "lnk" { continue; }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
        // Skip uninstallers
        let lower = stem.to_lowercase();
        if lower.contains("uninstall") || lower.contains("setup") { continue; }

        let name = stem.to_string();
        apps.push(AppEntry {
            id: format!("wine:{}", path.display()),
            name: name.clone(),
            exec: format!("wine start /unix \"{}\"", path.display()),
            icon_path: super::icon::fallback_icon(),
            source: AppSource::Wine,
            desktop_path: None,
            lower_name: lower,
            haystack: Utf32String::from(name.as_str()),
            keywords: vec!["wine".to_string(), "windows".to_string()],
            categories: vec!["Wine".to_string()],
            launch_count: 0,
            last_launched: None,
        });
    }
}
