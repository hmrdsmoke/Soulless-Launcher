// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use super::{AppEntry, AppSource};
use nucleo_matcher::Utf32String;
use std::fs;

pub fn index() -> Vec<AppEntry> {
    let mut apps = Vec::new();

    let home = dirs::home_dir().unwrap_or_default();

    let steamapps = format!(
        "{}/.local/share/Steam/steamapps",
        home.display()
    );

    let Ok(entries) = fs::read_dir(steamapps) else {
        return apps;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if !name.starts_with("appmanifest_") {
            continue;
        }

        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };

        let Some(appid) = extract_field(&contents, "appid") else {
            continue;
        };

        let Some(game_name) = extract_field(&contents, "name") else {
            continue;
        };

        let lower_name = game_name.to_lowercase();

        let icon_path = resolve_steam_icon(&appid);

        apps.push(AppEntry {
            id: format!("steam:{}", appid),
            name: game_name.clone(),
            exec: format!("steam steam://rungameid/{}", appid),
            icon_path,
            source: AppSource::Steam,
            desktop_path: None,
            lower_name,
            haystack: Utf32String::from(game_name.as_str()),
            keywords: vec!["steam".to_string(), "game".to_string()],
            categories: vec!["Games".to_string()],
            launch_count: 0,
            last_launched: None,
        });
    }

    apps
}

/// Resolves a Steam game icon from the librarycache folder.
///
/// Prefers header.jpg; falls back to the first jpg found in the folder.
fn resolve_steam_icon(appid: &str) -> String {
    let home = dirs::home_dir().unwrap_or_default();
    let cache_dir = home
        .join(".local/share/Steam/appcache/librarycache")
        .join(appid);

    // Prefer header.jpg — present on most games
    let header = cache_dir.join("header.jpg");
    if header.exists() {
        return header.display().to_string();
    }

    // Fall back to first jpg found in folder
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jpg") {
                return path.display().to_string();
            }
        }
    }

    super::icon::FALLBACK_ICON.to_string()
}

/// Extracts a quoted string value from a Steam .acf manifest field.
///
/// Example line: `"name"  "Portal 2"`  → returns `Some("Portal 2")`
fn extract_field(contents: &str, field: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();

        // Each line looks like: "key"  "value"
        let key_pattern = format!("\"{}\"", field);

        if !trimmed.starts_with(&key_pattern) {
            continue;
        }

        // Find the value after the key
        let after_key = &trimmed[key_pattern.len()..].trim_start();

        if after_key.starts_with('"') {
            let inner = &after_key[1..];
            if let Some(end) = inner.find('"') {
                return Some(inner[..end].to_string());
            }
        }
    }

    None
}

// === DONE ===
// Fixed: file was truncated mid-struct-literal :: done
// Added missing launch_count, last_launched fields :: done
// Added closing braces for struct, loop, and fn index() :: done
// Added extract_field() helper for .acf manifest parsing :: done