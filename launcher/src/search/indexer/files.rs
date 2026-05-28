// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use super::{AppEntry, AppSource};
use nucleo_matcher::Utf32String;
use std::fs;

pub fn index() -> Vec<AppEntry> {
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

    for dir in &xdg_dirs {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Skip hidden files
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if file_name.starts_with('.') {
                continue;
            }

            let name = file_name.to_string();
            let exec = format!("xdg-open {:?}", path);
            let lower_name = name.to_lowercase();
            let id = format!("file:{}", path.display());

            apps.push(AppEntry {
                id,
                name: name.clone(),
                exec,
                icon_path: String::new(),
                source: AppSource::File,
                lower_name: lower_name.clone(),
                haystack: Utf32String::from(name.as_str()),
                keywords: Vec::new(),
                categories: vec!["File".to_string()],
                launch_count: 0,
                last_launched: None,
            });
        }
    }

    apps
}
