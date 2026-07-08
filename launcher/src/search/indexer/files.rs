// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/search/indexer/files.rs
// Indexes files and folders under common home directories.

use super::{AppEntry, AppSource};
use super::icon::IconCache;
use nucleo_matcher::Utf32String;
use std::fs;
use std::path::Path;

pub fn index(icons: &mut IconCache) -> Vec<AppEntry> {
    let mut apps = Vec::new();

    let home = dirs::home_dir().unwrap_or_default();

    let xdg_dirs = [
        dirs::document_dir().unwrap_or_else(|| home.join("Documents")),
        dirs::download_dir().unwrap_or_else(|| home.join("Downloads")),
        dirs::desktop_dir().unwrap_or_else(|| home.join("Desktop")),
        dirs::picture_dir().unwrap_or_else(|| home.join("Pictures")),
        dirs::video_dir().unwrap_or_else(|| home.join("Videos")),
        dirs::audio_dir().unwrap_or_else(|| home.join("Music")),
    ];

    let folder_icon = icons.resolve(Some("folder"));
    let file_icon = icons.resolve(Some("text-x-generic"));

    for dir in &xdg_dirs {
        // Add the XDG dir itself as an entry
        if let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) {
            let name = dir_name.to_string();
            let lower_name = name.to_lowercase();
            apps.push(AppEntry {
                id: format!("file:{}", dir.display()),
                name: name.clone(),
                exec: format!("xdg-open {}", dir.display()),
                icon_path: folder_icon.clone(),
                source: AppSource::File,
                desktop_path: None,
                lower_name,
                haystack: Utf32String::from(name.as_str()),
                keywords: Vec::new(),
                categories: vec!["File".to_string()],
                launch_count: 0,
                last_launched: None,
            });
        }

        scan_dir(dir, &folder_icon, &file_icon, &mut apps);

        // One level deeper — scan subdirectories
        let Ok(entries) = fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !is_hidden(&path) {
                scan_dir(&path, &folder_icon, &file_icon, &mut apps);
            }
        }
    }

    // ── Full home scan with exclusions ──────────────────────────────────────
    let exclude = [
        "snap", "flatpak", ".var", "node_modules", "target",
        ".cargo", ".rustup", ".steam", ".local",
    ];
    let vault_dir = crate::vault::vault_dir();
    let Ok(home_entries) = fs::read_dir(&home) else {
        return apps;
    };
    for entry in home_entries.flatten() {
        let path = entry.path();
        if is_hidden(&path) { continue; }
        if path == vault_dir { continue; }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        if exclude.contains(&name.as_str()) { continue; }
        // Skip dirs already covered by XDG scan
        if xdg_dirs.contains(&path) { continue; }
        if path.is_dir() {
            scan_dir(&path, &folder_icon, &file_icon, &mut apps);
            let Ok(sub_entries) = fs::read_dir(&path) else { continue };
            for sub in sub_entries.flatten() {
                let sub_path = sub.path();
                if sub_path.is_dir() && !is_hidden(&sub_path) {
                    scan_dir(&sub_path, &folder_icon, &file_icon, &mut apps);
                }
            }
        }
    }
    apps
}

fn scan_dir(dir: &Path, folder_icon: &str, file_icon: &str, apps: &mut Vec<AppEntry>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };


    for entry in entries.flatten() {
        let path = entry.path();

        if is_hidden(&path) {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let is_dir = path.is_dir();
        let name = file_name.to_string();
        let lower_name = name.to_lowercase();
        let id = format!("file:{}", path.display());
        let exec = format!("xdg-open {}", path.display());

        let icon_path = if is_dir {
            folder_icon.to_string()
        } else {
            file_icon.to_string()
        };


        apps.push(AppEntry {
            id,
            name: name.clone(),
            exec,
            icon_path,
            source: AppSource::File,
            desktop_path: None,
            lower_name,
            haystack: Utf32String::from(name.as_str()),
            keywords: Vec::new(),
            categories: vec!["File".to_string()],
            launch_count: 0,
            last_launched: None,
        });
    }

}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}
