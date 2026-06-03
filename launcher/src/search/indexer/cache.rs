// MIT License - see LICENSE file for full terms
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
    let Ok(modified) = meta.modified() else {
        return true;
    };
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::MAX);
    age > Duration::from_secs(hours * 3600)
}
