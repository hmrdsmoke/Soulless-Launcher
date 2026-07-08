// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// organizer/src/scan.rs
// One-time startup scan of watched directories.

use crate::{PendingSuggestion, rules};

pub fn scan() -> Vec<PendingSuggestion> {
    let home = match dirs::home_dir() { Some(h) => h, None => return vec![] };
    let watch_dirs = vec![
        dirs::download_dir().unwrap_or_else(|| home.join("Downloads")),
        dirs::document_dir().unwrap_or_else(|| home.join("Documents")),
        dirs::picture_dir().unwrap_or_else(|| home.join("Pictures")),
        dirs::video_dir().unwrap_or_else(|| home.join("Videos")),
        dirs::audio_dir().unwrap_or_else(|| home.join("Music")),
        home.join("Desktop"),
    ];
    let mut suggestions = Vec::new();
    for dir in watch_dirs {
        if !dir.exists() { continue; }
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() { continue; }
            if let Some(suggestion) = rules::suggest(&path) {
            suggestions.push(PendingSuggestion { suggestion });
            }
        }
    }
    suggestions
}
