// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/search/indexer/jetbrains.rs
// Indexes JetBrains IDEs installed via JetBrains Toolbox.

use super::{AppEntry, AppSource};
use nucleo_matcher::Utf32String;
use std::fs;
use std::path::Path;

pub fn index() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // JetBrains Toolbox installs to ~/.local/share/JetBrains/Toolbox/apps/
    let toolbox_dir = home.join(".local/share/JetBrains/Toolbox/apps");
    if toolbox_dir.exists() {
        scan_toolbox(&toolbox_dir, &mut apps);
    }

    // Also check ~/.local/share/applications for JetBrains .desktop files
    let apps_dir = home.join(".local/share/applications");
    if apps_dir.exists() {
        scan_desktop_files(&apps_dir, &mut apps);
    }

    apps
}

fn scan_toolbox(dir: &Path, apps: &mut Vec<AppEntry>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }
        // Each app dir contains ch-0/bin/<appname>.sh or similar
        let ch_dir = path.join("ch-0");
        let bin_dir = ch_dir.join("bin");
        if !bin_dir.exists() { continue; }
        let Ok(bins) = fs::read_dir(&bin_dir) else { continue };
        for bin in bins.flatten() {
            let bp = bin.path();
            let ext = bp.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "sh" { continue; }
            let Some(app_name) = bp.file_stem().and_then(|s| s.to_str()) else { continue };
            let name = format_name(app_name);
            let lower = name.to_lowercase();
            apps.push(AppEntry {
                id: format!("jetbrains:{}", app_name),
                name: name.clone(),
                exec: bp.display().to_string(),
                icon_path: super::icon::fallback_icon(),
                source: AppSource::JetBrains,
                lower_name: lower,
                haystack: Utf32String::from(name.as_str()),
                keywords: vec!["jetbrains".to_string(), "ide".to_string()],
                categories: vec!["Development".to_string()],
                launch_count: 0,
                last_launched: None,
            });
        }
    }
}

fn scan_desktop_files(dir: &Path, apps: &mut Vec<AppEntry>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.contains("jetbrains") && !name.contains("JetBrains") { continue; }
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") { continue; }
        let Ok(contents) = fs::read_to_string(&path) else { continue };
        let app_name = extract_desktop_field(&contents, "Name").unwrap_or_default();
        let exec = extract_desktop_field(&contents, "Exec").unwrap_or_default();
        let icon = extract_desktop_field(&contents, "Icon").unwrap_or_default();
        if app_name.is_empty() || exec.is_empty() { continue; }
        let lower = app_name.to_lowercase();
        apps.push(AppEntry {
            id: format!("jetbrains-desktop:{}", app_name),
            name: app_name.clone(),
            exec,
            icon_path: if icon.is_empty() { super::icon::fallback_icon() } else { icon },
            source: AppSource::JetBrains,
            lower_name: lower,
            haystack: Utf32String::from(app_name.as_str()),
            keywords: vec!["jetbrains".to_string(), "ide".to_string()],
            categories: vec!["Development".to_string()],
            launch_count: 0,
            last_launched: None,
        });
    }
}

fn extract_desktop_field(contents: &str, field: &str) -> Option<String> {
    let prefix = format!("{}=", field);
    contents.lines()
        .find(|l| l.starts_with(&prefix))
        .map(|l| l[prefix.len()..].trim().to_string())
}

fn format_name(raw: &str) -> String {
    // "idea" -> "IntelliJ IDEA", "pycharm" -> "PyCharm", etc.
    match raw.to_lowercase().as_str() {
        "idea" => "IntelliJ IDEA".to_string(),
        "pycharm" => "PyCharm".to_string(),
        "webstorm" => "WebStorm".to_string(),
        "clion" => "CLion".to_string(),
        "goland" => "GoLand".to_string(),
        "rider" => "Rider".to_string(),
        "datagrip" => "DataGrip".to_string(),
        "rubymine" => "RubyMine".to_string(),
        "phpstorm" => "PhpStorm".to_string(),
        "rustrover" => "RustRover".to_string(),
        "fleet" => "Fleet".to_string(),
        _ => raw.to_string(),
    }
}
