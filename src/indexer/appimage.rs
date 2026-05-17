// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use crate::indexer::{AppEntry, AppSource};
use crate::indexer::icon::IconCache;
use nucleo_matcher::Utf32String;
use std::fs;

pub fn index(icons: &mut IconCache) -> Vec<AppEntry> {
    let mut apps = Vec::new();

    let home = dirs::home_dir().unwrap_or_default();

    let search_dirs = [
        format!("{}/Applications", home.display()),
        format!("{}/AppImages", home.display()),
        format!("{}/Downloads", home.display()),
    ];

    for dir in search_dirs {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("AppImage") {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };

            let name = stem.replace('_', " ");
            let lower_name = name.to_lowercase();

            // Try to resolve an icon by the app name; falls back to launcher.png
            let icon_path = icons.resolve(Some(stem));

            apps.push(AppEntry {
                id: format!("appimage:{}", path.display()),
                name: name.clone(),
                exec: path.display().to_string(),
                icon_path,
                source: AppSource::AppImage,
                lower_name,
                haystack: Utf32String::from(name.as_str()),
                keywords: vec!["appimage".to_string()],
                categories: vec!["Portable".to_string()],
                launch_count: 0,
                last_launched: None,
            });
        }
    }

    apps
}

// === DONE ===
// Wired IconCache — icons resolved once at startup :: done
// AppImage icon lookup tries stem name before falling back :: done