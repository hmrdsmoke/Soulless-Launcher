// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/config/mod.rs
// User configuration for the Soulless launcher.
// Config lives at ~/.config/soulless/config.ron

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

/// Ensure config directories exist
#[allow(dead_code)]
pub fn ensure_dirs() {
    let _ = std::fs::create_dir_all(backgrounds_dir());
}

/// Returns the path of the first background image found,
/// or None if no backgrounds are installed.
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

/// Load and resize background image to exact panel dimensions.
/// Returns RGBA bytes sized to (width x height).
pub fn load_background_rgba(path: &str, width: u32, height: u32) -> Option<Vec<u8>> {
    let img = image::open(path).ok()?;
    let resized = img.resize_to_fill(width, height, image::imageops::FilterType::Lanczos3);
    Some(resized.to_rgba8().into_raw())
}
