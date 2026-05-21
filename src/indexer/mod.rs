// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use nucleo_matcher::Utf32String;

pub mod appimage;
pub mod cache;
pub mod desktop;
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
}

use crate::indexer::icon::IconCache;

pub fn build_index() -> Vec<AppEntry> {
    let mut apps = Vec::new();

    // Shared startup icon resolver
    let mut icons = IconCache::new();

    // Source priority: Desktop > Flatpak > AppImage > Steam
    // PATH binaries are excluded — too noisy for a GUI launcher
    apps.extend(desktop::index(&mut icons));
    apps.extend(flatpak::index(&mut icons));
    apps.extend(appimage::index(&mut icons));
    apps.extend(steam::index());

    // Deduplicate by lowercase name — keeps the highest-priority entry.
    // Removes duplicates like an app appearing as both .desktop and Flatpak.
    let mut seen_names = std::collections::HashSet::new();
    apps.retain(|app| seen_names.insert(app.lower_name.clone()));

    apps.sort_by(|a, b| a.name.cmp(&b.name));

    apps
}