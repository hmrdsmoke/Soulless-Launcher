// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use super::AppEntry;
use nucleo_matcher::Utf32String;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("soulless")
}

fn cache_path(source: &str) -> PathBuf {
    cache_dir().join(format!("{}.bin", source))
}

pub fn save(source: &str, apps: &[AppEntry]) {
    let path = cache_path(source);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match bincode::serialize(apps) {
        Ok(bytes) => {
            if let Err(e) = fs::write(&path, bytes) {
                eprintln!("cache: failed to write {}: {}", path.display(), e);
            }
        }
        Err(e) => eprintln!("cache: serialize error for {}: {}", source, e),
    }
}

pub fn invalidate(source: &str) {
    let path = cache_path(source);
    let _ = fs::remove_file(path);
}

pub fn load(source: &str) -> Option<Vec<AppEntry>> {
    let path = cache_path(source);
    let bytes = fs::read(&path).ok()?;
    let mut apps: Vec<AppEntry> = bincode::deserialize(&bytes).ok()?;
    // Rebuild haystack — skipped during serde
    for app in &mut apps {
        app.haystack = Utf32String::from(app.name.as_str());
    }
    Some(apps)
}

pub fn is_stale(source: &str, hours: u64) -> bool {
    let path = cache_path(source);
    let Ok(meta) = fs::metadata(&path) else {
        return true; // missing = stale
    };
    let Ok(cache_modified) = meta.modified() else {
        return true;
    };

    // Time-based ceiling: rebuild at least every `hours`.
    let age = SystemTime::now()
        .duration_since(cache_modified)
        .unwrap_or(Duration::MAX);
    if age > Duration::from_secs(hours * 3600) {
        return true;
    }

    // Change-based: if any application directory for this source has been
    // modified more recently than the cache, an app was installed/removed —
    // rebuild so new apps show up immediately (don't wait out the window).
    for dir in source_dirs(source) {
        if let Ok(dir_meta) = fs::metadata(&dir)
            && let Ok(dir_modified) = dir_meta.modified()
                && dir_modified > cache_modified {
                    return true;
                }
    }

    false
}

/// The directories whose changes should invalidate a given source's cache.
/// Installing/removing an app updates its directory's mtime, so comparing
/// these against the cache's mtime detects new installs without a 24h wait.
fn source_dirs(source: &str) -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    match source {
        "desktop" => {
            let mut v = Vec::new();
            let xdg = std::env::var("XDG_DATA_DIRS")
                .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
            for d in xdg.split(':') {
                v.push(PathBuf::from(format!("{}/applications", d)));
            }
            v.push(home.join(".local/share/applications"));
            v
        }
        "flatpak" => vec![
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
            home.join(".local/share/flatpak/exports/share/applications"),
        ],
        _ => Vec::new(), // other sources keep time-based-only for now
    }
}
