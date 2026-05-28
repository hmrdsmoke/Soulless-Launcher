// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use super::{AppEntry, AppSource};
use super::icon::IconCache;
use nucleo_matcher::Utf32String;
use std::fs;
use std::path::{Path, PathBuf};

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
                lower_name,
                haystack: Utf32String::from(name.as_str()),
                keywords: Vec::new(),
                categories: vec!["File".to_string()],
                launch_count: 0,
                last_launched: None,
            });
        }

        scan_dir(dir, &folder_icon, &mut apps);

        // One level deeper — scan subdirectories
        let Ok(entries) = fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !is_hidden(&path) {
                scan_dir(&path, &folder_icon, &mut apps);
            }
        }
    }

    eprintln!("files: indexed {} files across XDG dirs", apps.len());
    apps
}

fn scan_dir(dir: &Path, folder_icon: &str, apps: &mut Vec<AppEntry>) {
    let Ok(entries) = fs::read_dir(dir) else {
        eprintln!("files: skipping missing dir {:?}", dir);
        return;
    };

    let mut file_count = 0usize;
    let mut dir_count = 0usize;

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
            dir_count += 1;
            folder_icon.to_string()
        } else {
            file_count += 1;
            super::icon::fallback_icon()
        };

        eprintln!("files: adding {} {:?} exec={:?}", if is_dir { "dir" } else { "file" }, name, exec);

        apps.push(AppEntry {
            id,
            name: name.clone(),
            exec,
            icon_path,
            source: AppSource::File,
            lower_name,
            haystack: Utf32String::from(name.as_str()),
            keywords: Vec::new(),
            categories: vec!["File".to_string()],
            launch_count: 0,
            last_launched: None,
        });
    }

    eprintln!("files: {} files, {} dirs indexed in {:?}", file_count, dir_count, dir);
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}
