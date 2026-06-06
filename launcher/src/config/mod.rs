// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/config/mod.rs
// User configuration for the Soulless launcher.
// Config lives at ~/.config/soulless/config.ron

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Returns ~/.config/soulless/
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("soulless")
}

/// Returns ~/.config/soulless/backgrounds/
pub fn backgrounds_dir() -> PathBuf {
    config_dir().join("backgrounds")
}

/// Returns ~/.config/soulless/config.ron
pub fn config_path() -> PathBuf {
    config_dir().join("config.ron")
}

/// Ensure config directories exist
pub fn ensure_dirs() {
    let _ = std::fs::create_dir_all(backgrounds_dir());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeVariant {
    Dark,
    Chrome,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        ThemeVariant::Chrome
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoullessConfig {
    /// Show system/hardware/network/fps monitor widgets
    pub show_system_monitor: bool,
    /// How deep to scan home directory for files (1 = XDG dirs only, 2 = one level deeper)
    pub search_file_depth: u32,
    /// Enable the file organizer
    pub organizer_enabled: bool,
    /// Directories the organizer watches (empty = use defaults)
    pub organizer_watch_dirs: Vec<PathBuf>,
    /// App icon size in the grid
    pub drawer_icon_size: f32,
    /// Theme variant
    pub theme_variant: ThemeVariant,
}

impl Default for SoullessConfig {
    fn default() -> Self {
        Self {
            show_system_monitor: true,
            search_file_depth: 2,
            organizer_enabled: true,
            organizer_watch_dirs: vec![],
            drawer_icon_size: 64.0,
            theme_variant: ThemeVariant::Chrome,
        }
    }
}

/// Load config from disk, returning defaults if not found or invalid.
pub fn load_config() -> SoullessConfig {
    let path = config_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return SoullessConfig::default();
    };
    ron::from_str(&text).unwrap_or_default()
}

/// Save config to disk.
pub fn save_config(config: &SoullessConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let pretty = ron::ser::to_string_pretty(config, ron::ser::PrettyConfig::default())
        .unwrap_or_default();
    let _ = std::fs::write(&path, pretty);
}

/// Write default config if none exists.
pub fn ensure_config() {
    if !config_path().exists() {
        save_config(&SoullessConfig::default());
    }
}

/// Load and resize background image to exact panel dimensions.
pub fn load_background_rgba(path: &str, width: u32, height: u32) -> Option<Vec<u8>> {
    let img = image::open(path).ok()?;
    let resized = img.resize_to_fill(width, height, image::imageops::FilterType::Lanczos3);
    Some(resized.to_rgba8().into_raw())
}

/// Returns the path of the first background image found.
pub fn default_background() -> Option<String> {
    let dir = backgrounds_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return None;
    };
    let supported = ["jpg", "jpeg", "png", "webp"];
    let mut images: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| supported.contains(&e.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    images.sort();
    images.first().map(|p| p.display().to_string())
}
