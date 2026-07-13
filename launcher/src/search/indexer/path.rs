// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI) and Claude (Anthropic).
// Do not remove these comments.
// launcher/src/search/indexer/path.rs
// Indexes executables found on the user's PATH.

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
        "sudo", "mount", "umount", "sh", "bash", "dash", "zsh",
        "python", "python3", "perl", "ruby", "node",
    ];

    // In-sandbox, PATH is the runtime's — zero host tools. Walk the host's
    // real bin dirs instead (mounted read-only at /run/host/usr).
    use crate::search::indexer::hostpath;

    let path_env = if hostpath::sandboxed() {
        [
            hostpath::host("/usr/bin"),
            hostpath::host("/usr/local/bin"),
            hostpath::host("/bin"),
            hostpath::host("/usr/sbin"),
        ]
        .join(":")
    } else {
        std::env::var("PATH").unwrap_or_default()
    };

    for dir in std::env::split_paths(&path_env) {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // NOTE: in-sandbox, /run/host/usr/bin is full of symlinks whose
            // targets (e.g. /etc/alternatives/*) don't resolve inside the
            // sandbox root. is_file() FOLLOWS symlinks, so it returned false
            // for nearly every host binary. symlink_metadata() stats the link
            // itself and doesn't follow.
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                continue;
            };

            if metadata.is_dir() {
                continue;
            }

            // Executable bit: check the link's own mode, falling back to the
            // target's when the link resolves (native case).
            let mode = if metadata.file_type().is_symlink() {
                fs::metadata(&path)
                    .map(|m| m.permissions().mode())
                    .unwrap_or(0o755) // unresolvable in-sandbox: trust the bin dir
            } else {
                metadata.permissions().mode()
            };

            if mode & 0o111 == 0 {
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
                // CLI tools open their --help in a terminal pager on click.
                exec: format!("cosmic-term -e sh -c '{} --help 2>&1 | less'", name),
                // CLI tools all share the utilities (wrench) icon instead of a
                // mostly-failing per-command name lookup.
                icon_path: icons.resolve(Some("applications-utilities")),
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
// symlink_metadata: host bin symlinks don't resolve in-sandbox :: done
