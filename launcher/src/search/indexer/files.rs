// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// ── File source contract ──────────────────────────────────────────────────────
//
// Deliberately shallow. The old everything-index walked home two levels deep
// and materialized every file it found — unbounded entry count held for the
// daemon's life, rescanned per keystroke, rebuilt synchronously on the
// activation path. The index is for *launching points*, not a file manager:
//
//   1. The six XDG dirs themselves (Documents, Downloads, ...).
//   2. Everything at the TOP LEVEL of each XDG dir — files and folders.
//   3. Folders (only) at the top level of home.
//
// No recursion anywhere. Seven readdirs total, entry count bounded by what
// a human keeps at their top levels. Deeper navigation is one xdg-open away.

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

    // ── 1 + 2: each XDG dir itself, then its top level ───────────────────────
    for dir in &xdg_dirs {
        push_entry(dir, true, &folder_icon, &file_icon, &mut apps);
        scan_top(dir, true, &folder_icon, &file_icon, &mut apps);
    }

    // ── 3: folders at the top level of home ──────────────────────────────────
    let exclude = [
        "snap", "flatpak", "node_modules", "target",
    ];
    let vault_dir = crate::vault::vault_dir();
    let Ok(home_entries) = fs::read_dir(&home) else {
        return apps;
    };
    for entry in home_entries.flatten() {
        let path = entry.path();
        if is_hidden(&path) { continue; }
        if path == vault_dir { continue; }
        if !path.is_dir() { continue; }          // folders only at home level
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if exclude.contains(&name) { continue; }
        // XDG dirs already added above
        if xdg_dirs.contains(&path) { continue; }
        push_entry(&path, true, &folder_icon, &file_icon, &mut apps);
    }

    apps
}

/// Index every non-hidden entry at the top level of `dir` — files and
/// folders if `include_files`, folders only otherwise. Never recurses.
fn scan_top(
    dir: &Path,
    include_files: bool,
    folder_icon: &str,
    file_icon: &str,
    apps: &mut Vec<AppEntry>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if is_hidden(&path) {
            continue;
        }
        let is_dir = path.is_dir();
        if !is_dir && !include_files {
            continue;
        }
        push_entry(&path, is_dir, folder_icon, file_icon, apps);
    }
}

fn push_entry(
    path: &Path,
    is_dir: bool,
    folder_icon: &str,
    file_icon: &str,
    apps: &mut Vec<AppEntry>,
) {
    let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
        return;
    };
    let name = file_name.to_string();
    let lower_name = name.to_lowercase();

    apps.push(AppEntry {
        id: format!("file:{}", path.display()),
        name: name.clone(),
        exec: format!("xdg-open {}", path.display()),
        icon_path: if is_dir { folder_icon.to_string() } else { file_icon.to_string() },
        source: if is_dir { AppSource::Folder } else { AppSource::File },
        desktop_path: None,
        lower_name,
        haystack: Utf32String::from(name.as_str()),
        keywords: Vec::new(),
        categories: vec!["File".to_string()],
        launch_count: 0,
        last_launched: None,
    });
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}