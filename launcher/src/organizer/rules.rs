// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/organizer/rules.rs
// Rule engine for classifying files and suggesting destinations.
// Add new rules here — no AI needed, just smart pattern matching.

use std::path::{Path, PathBuf};

/// A suggestion for where a file should be moved.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MoveSuggestion {
    pub from: PathBuf,
    pub to: PathBuf,
    pub reason: String,
}

/// Classify a file and suggest where it should live.
/// Returns None if no rule matches.
pub fn suggest(path: &Path) -> Option<MoveSuggestion> {
    let home = dirs::home_dir()?;
    let name = path.file_name()?.to_str()?.to_lowercase();
    let ext = path.extension()?.to_str()?.to_lowercase();

    // Already in the right place
    let downloads = dirs::download_dir().unwrap_or_else(|| home.join("Downloads"));
    if !path.starts_with(&downloads) {
        return None;
    }

    let dest: Option<PathBuf> = match ext.as_str() {
        // Images
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "tiff" | "heic" => {
            Some(dirs::picture_dir().unwrap_or_else(|| home.join("Pictures")))
        }
        // Videos
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" => {
            Some(dirs::video_dir().unwrap_or_else(|| home.join("Videos")))
        }
        // Audio
        "mp3" | "flac" | "wav" | "ogg" | "aac" | "m4a" => {
            Some(dirs::audio_dir().unwrap_or_else(|| home.join("Music")))
        }
        // Documents
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "txt" | "md" => {
            Some(dirs::document_dir().unwrap_or_else(|| home.join("Documents")))
        }
        // Spreadsheets
        "xls" | "xlsx" | "ods" | "csv" => {
            Some(dirs::document_dir().unwrap_or_else(|| home.join("Documents")))
        }
        // Presentations
        "ppt" | "pptx" | "odp" => {
            Some(dirs::document_dir().unwrap_or_else(|| home.join("Documents")))
        }
        // AppImages stay in Downloads or go to ~/Applications
        "appimage" => {
            Some(home.join("Applications"))
        }
        // Archives — stay in Downloads, they need to be extracted
        "zip" | "tar" | "gz" | "xz" | "bz2" | "7z" | "rar" => {
            return None;
        }
        _ => None,
    };

    // Name-based overrides
    let dest = dest.or_else(|| {
        if name.contains("resume") || name.contains("cv") || name.contains("curriculum") {
            Some(home.join("Documents").join("Career"))
        } else if name.contains("invoice") || name.contains("receipt") || name.contains("tax") {
            Some(home.join("Documents").join("Finance"))
        } else if name.contains("screenshot") || name.starts_with("screen") {
            Some(dirs::picture_dir().unwrap_or_else(|| home.join("Pictures")).join("Screenshots"))
        } else {
            None
        }
    })?;

    let dest_file = dest.join(path.file_name()?);
    let reason = format!(
        "{} looks like it belongs in {}",
        path.file_name()?.to_str()?,
        dest.display()
    );

    Some(MoveSuggestion {
        from: path.to_path_buf(),
        to: dest_file,
        reason,
    })
}
