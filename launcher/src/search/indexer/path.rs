// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use super::{AppEntry, AppSource};
use super::icon::IconCache;
use nucleo_matcher::Utf32String;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;

pub fn index(icons: &mut IconCache) -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let mut seen = HashSet::new();

    let blacklist = [
        "sudo",
        "mount",
        "umount",
        "sh",
        "bash",
        "dash",
        "zsh",
        "python",
        "python3",
        "perl",
        "ruby",
        "node",
    ];

    let path_env = std::env::var("PATH").unwrap_or_default();

    for dir in std::env::split_paths(&path_env) {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let Ok(metadata) = fs::metadata(&path) else {
                continue;
            };

            // Skip non-executable files
            if metadata.permissions().mode() & 0o111 == 0 {
                continue;
            }

            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            if blacklist.contains(&name) {
                continue;
            }

            // Skip duplicates (first PATH entry wins)
            if !seen.insert(name.to_string()) {
                continue;
            }

            let lower_name = name.to_lowercase();

            apps.push(AppEntry {
                id: format!("binary:{}", path.display()),
                name: name.to_string(),
                exec: path.display().to_string(),
                icon_path: icons.resolve(Some(name)),
                source: AppSource::Binary,
                desktop_path: None,
                lower_name,
                haystack: Utf32String::from(name),
                keywords: Vec::new(),
                categories: vec!["Binary".to_string()],
                launch_count: 0,
                last_launched: None,
            });
        }
    }

    apps
}

// === DONE ===
// Fixed: loop body was outside function — moved inside fn index() :: done
// Added missing use std::os::unix::fs::PermissionsExt for .mode() :: done
// Added missing let path_env = std::env::var("PATH") :: done
// Added missing nucleo_matcher::Utf32String import :: done
// Extended blacklist with common shells and runtimes :: done