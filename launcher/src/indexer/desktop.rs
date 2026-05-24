// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use crate::indexer::icon::IconCache;
use crate::indexer::{AppEntry, AppSource};

use freedesktop_desktop_entry::DesktopEntry;
use nucleo_matcher::Utf32String;

use std::fs;

pub fn index(
    icons: &mut IconCache,
) -> Vec<AppEntry> {
    let mut apps = Vec::new();

    let home = dirs::home_dir().unwrap_or_default();

    let dirs = [
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
        format!(
            "{}/.local/share/applications",
            home.display()
        ),
    ];

    for dir in dirs {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // ONLY .desktop files
            if path.extension().and_then(|s| s.to_str())
                != Some("desktop")
            {
                continue;
            }

            let Ok(desktop) =
                DesktopEntry::from_path::<&str>(
                    path.clone(),
                    &[],
                )
            else {
                continue;
            };

            let Some(name) =
                desktop.name::<&str>(&[])
            else {
                continue;
            };

            let Some(exec) = desktop.exec()
            else {
                continue;
            };

            // Skip hidden launcher entries
            if desktop.no_display() {
                continue;
            }

            if should_skip_entry(exec, &name) {
                continue;
            }

            // Resolve icon ONCE during indexing
            let icon_path = icons.resolve(
                desktop.icon()
            );

            let clean_exec =
                strip_desktop_placeholders(exec);

            let name_str = name.to_string();

            let lower_name =
                name_str.to_lowercase();

            let id = format!(
                "desktop:{}",
                path.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or(&name_str)
            );

            apps.push(AppEntry {
                id,

                name: name_str.clone(),

                exec: clean_exec,

                icon_path,

                source: AppSource::Desktop,

                lower_name,

                haystack:
                    Utf32String::from(
                        name_str.as_str()
                    ),

                keywords: Vec::new(),

                categories: Vec::new(),

                launch_count: 0,

                last_launched: None,
            });
        }
    }

    apps
}

fn should_skip_entry(exec: &str, name: &str) -> bool {
    let lower_exec = exec.to_lowercase();
    let lower_name = name.to_lowercase();

    // Skip background services, handlers, and non-launchable entries
    let exec_skip = [
        "handler", "oauth", "daemon", "service", "portal",
        "agent", "polkit", "pkexec", "gksu", "kdesu",
    ];

    // Skip entries whose names indicate they are settings panels,
    // system components, or duplicates of things already in the desktop index
    let name_skip = [
        "settings panel",
        "control center module",
        "system settings module",
    ];

    exec_skip.iter().any(|t| lower_exec.contains(t))
        || name_skip.iter().any(|t| lower_name.contains(t))
}

fn strip_desktop_placeholders(
    exec: &str,
) -> String {
    let mut result =
        String::with_capacity(exec.len());

    let mut chars =
        exec.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            if chars
                .peek()
                .map_or(
                    false,
                    |&n| n.is_ascii_alphabetic(),
                )
            {
                chars.next();
                continue;
            }
        }

        result.push(c);
    }

    result.trim().to_string()
}

// === DONE ===
// Wired IconCache into desktop indexer :: done
// Icons now resolved exactly once at startup :: done
// Rendering-time icon filesystem scans eliminated :: done
// NoDisplay=true filtering preserved :: done
// Placeholder stripping preserved :: done
// Added stable icon_path architecture :: done
// === DONE ===
// Fixed: AppEntry struct literal was missing 6 fields :: done
// Added lower_name, haystack, keywords, categories, launch_count, last_launched :: done
// Added should_skip_entry and strip_desktop_placeholders helpers :: done
// Added NoDisplay=true filter to skip invisible entries :: done