// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use super::icon::IconCache;
use super::{AppEntry, AppSource};

use freedesktop_desktop_entry::DesktopEntry;
use nucleo_matcher::Utf32String;

use std::fs;

pub fn index(
    icons: &mut IconCache,
) -> Vec<AppEntry> {
    let mut apps = Vec::new();

    let home = dirs::home_dir()
        .unwrap_or_default();

    let dirs = [
        "/var/lib/flatpak/exports/share/applications"
            .to_string(),

        format!(
            "{}/.local/share/flatpak/exports/share/applications",
            home.display()
        ),
    ];

    for dir in dirs {
        let Ok(entries) =
            fs::read_dir(dir)
        else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // ONLY .desktop files
            if path
                .extension()
                .and_then(|s| s.to_str())
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

            let Some(exec) =
                desktop.exec()
            else {
                continue;
            };

            // Resolve icon ONCE during indexing
            let icon_path = icons.resolve(
                desktop.icon()
            );

            let clean_exec =
                strip_desktop_placeholders(
                    exec
                );

            let lower_name =
                name.to_lowercase();

            let id = format!(
                "flatpak:{}",
                path.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or(&name)
            );

            apps.push(AppEntry {
                id,

                name: name.to_string(),

                exec: clean_exec,

                icon_path,

                source: AppSource::Flatpak,
                desktop_path: None,

                lower_name,

                haystack:
                    Utf32String::from(name),

                keywords: vec![
                    "flatpak".to_string(),
                ],

                categories: vec![
                    "Flatpak".to_string(),
                ],

                launch_count: 0,

                last_launched: None,
            });
        }
    }

    apps
}

fn strip_desktop_placeholders(
    exec: &str,
) -> String {
    exec
        .replace("%u", "")
        .replace("%U", "")
        .replace("%f", "")
        .replace("%F", "")
        .replace("%i", "")
        .replace("%c", "")
        .replace("%k", "")
        .trim()
        .to_string()
}

// === DONE ===
// Wired IconCache into flatpak indexer :: done
// Icons resolved once during startup :: done
// Rendering-time icon lookups eliminated :: done
// Stable icon_path architecture implemented :: done