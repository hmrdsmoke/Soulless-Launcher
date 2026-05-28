// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use nucleo_matcher::Utf32String;

pub mod appimage;
pub mod cache;
pub mod desktop;
pub mod files;
pub mod flatpak;
pub mod path;
pub mod steam;
pub mod icon;

#[derive(Clone)]
pub struct AppEntry {
    pub id: String,

    pub name: String,
    pub exec: String,

    pub icon_path: String,

    pub source: AppSource,

    pub lower_name: String,
    pub haystack: Utf32String,

    pub keywords: Vec<String>,
    pub categories: Vec<String>,

    pub launch_count: u32,
    pub last_launched: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppSource {
    Desktop,
    Flatpak,
    Steam,
    AppImage,
    Binary,
    Script,
    JetBrains,
    Wine,
    Proton,
    File,
}

use crate::search::indexer::icon::IconCache;

pub fn build_index() -> Vec<AppEntry> {
    let mut apps = Vec::new();

    // Shared startup icon resolver
    let mut icons = IconCache::new();

    // Source priority: Desktop > Flatpak > AppImage > Steam > PATH binaries > Files
    apps.extend(desktop::index(&mut icons));
    apps.extend(flatpak::index(&mut icons));
    apps.extend(appimage::index(&mut icons));
    apps.extend(steam::index());

    // Only index binaries that have a man page (filters out noise)
    let path_apps: Vec<AppEntry> = path::index(&mut icons)
        .into_iter()
        .filter(|app| {
            let name = &app.name;
            std::path::Path::new(&format!("/usr/share/man/man1/{}.1.gz", name)).exists()
                || std::path::Path::new(&format!("/usr/share/man/man1/{}.1", name)).exists()
                || std::path::Path::new(&format!("/usr/share/man/man8/{}.8.gz", name)).exists()
        })
        .collect();
    apps.extend(path_apps);

    apps.extend(files::index());

    // Deduplicate by lowercase name — keeps the highest-priority entry.
    // Removes duplicates like an app appearing as both .desktop and Flatpak.
    let mut seen_names = std::collections::HashSet::new();
    apps.retain(|app| seen_names.insert(app.lower_name.clone()));

    apps.sort_by(|a, b| a.name.cmp(&b.name));

    apps
}