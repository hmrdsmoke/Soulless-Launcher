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
pub mod jetbrains;
pub mod wine;
pub mod appid;
pub mod icon;

#[allow(dead_code)] // issue #6 — launch stats tracking
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AppEntry {
    pub id: String,

    pub name: String,
    pub exec: String,

    pub icon_path: String,

    pub source: AppSource,

    pub lower_name: String,
    #[serde(skip)]
    pub haystack: Utf32String,

    pub keywords: Vec<String>,
    pub categories: Vec<String>,

    pub launch_count: u32,
    pub last_launched: Option<u64>,
    /// Path to the source .desktop file, if this app has one (user apps).
    /// Used to hide the app into the vault. None for non-.desktop sources.
    #[serde(default)]
    pub desktop_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    let mut icons = IconCache::new();

    icons.prewarm();

    // Each source cached independently for 24h. Rebuild + save on stale/miss.
    let desktop_apps = cache::load("desktop")
        .filter(|_| !cache::is_stale("desktop", 24))
        .unwrap_or_else(|| {
            let r = desktop::index(&mut icons);
            cache::save("desktop", &r);
            r
        });
    apps.extend(desktop_apps);

    let flatpak_apps = cache::load("flatpak")
        .filter(|_| !cache::is_stale("flatpak", 24))
        .unwrap_or_else(|| {
            let r = flatpak::index(&mut icons);
            cache::save("flatpak", &r);
            r
        });
    apps.extend(flatpak_apps);

    let snap_apps = cache::load("snap")
        .filter(|_| !cache::is_stale("snap", 24))
        .unwrap_or_else(|| {
            let r = appimage::index(&mut icons);
            cache::save("snap", &r);
            r
        });
    apps.extend(snap_apps);

    let steam_apps = cache::load("steam")
        .filter(|_| !cache::is_stale("steam", 24))
        .unwrap_or_else(|| {
            let r = steam::index();
            cache::save("steam", &r);
            r
        });
    apps.extend(steam_apps);

    let jetbrains_apps = cache::load("jetbrains")
        .filter(|_| !cache::is_stale("jetbrains", 24))
        .unwrap_or_else(|| {
            let r = jetbrains::index();
            cache::save("jetbrains", &r);
            r
        });
    apps.extend(jetbrains_apps);

    let wine_apps = cache::load("wine")
        .filter(|_| !cache::is_stale("wine", 24))
        .unwrap_or_else(|| {
            let r = wine::index();
            cache::save("wine", &r);
            r
        });
    apps.extend(wine_apps);

    let cli_apps = cache::load("cli")
        .filter(|_| !cache::is_stale("cli", 24))
        .unwrap_or_else(|| {
            let r: Vec<AppEntry> = path::index(&mut icons)
                .into_iter()
                .filter(|app| {
                    let name = &app.name;
                    // Only man1 — user commands; skip man8 system admin tools
                    let has_man =
                        std::path::Path::new(&format!("/usr/share/man/man1/{}.1.gz", name)).exists()
                        || std::path::Path::new(&format!("/usr/share/man/man1/{}.1", name)).exists();
                    // Extra noise filter
                    let not_noise = name.len() > 2
                        && !name.starts_with("x86_64")
                        && !name.starts_with("arm")
                        && !name.contains("config")
                        && !name.ends_with("-config");
                    has_man && not_noise
                })
                .collect();
            cache::save("cli", &r);
            r
        });
    let cli_count = cli_apps.len();
    apps.extend(cli_apps);

    // Files always rebuild fresh — no cache
    let file_apps = files::index(&mut icons);
    let file_count = file_apps.len();
    let app_count = apps.len();
    apps.extend(file_apps);

    eprintln!("Index loaded: {} apps, {} cli, {} files", app_count, cli_count, file_count);

    // Deduplicate by lowercase name — keeps the highest-priority entry.
    let mut seen_names = std::collections::HashSet::new();
    apps.retain(|app| seen_names.insert(app.lower_name.clone()));

    apps.sort_by(|a, b| a.name.cmp(&b.name));

    apps
}