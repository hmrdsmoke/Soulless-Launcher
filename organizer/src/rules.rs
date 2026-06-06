// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// organizer/src/rules.rs
// Rule engine for classifying files and suggesting destinations.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MoveSuggestion {
    pub from: PathBuf,
    pub to: PathBuf,
    pub reason: String,
}

pub fn suggest(path: &Path) -> Option<MoveSuggestion> {
    let home = dirs::home_dir()?;
    let name = path.file_name()?.to_str()?.to_lowercase();
    let ext = path.extension()?.to_str()?.to_lowercase();

    let downloads = dirs::download_dir().unwrap_or_else(|| home.join("Downloads"));

    let docs = dirs::document_dir().unwrap_or_else(|| home.join("Documents"));
    let pics = dirs::picture_dir().unwrap_or_else(|| home.join("Pictures"));
    let vids = dirs::video_dir().unwrap_or_else(|| home.join("Videos"));
    let music = dirs::audio_dir().unwrap_or_else(|| home.join("Music"));

    let dest: Option<PathBuf> = match ext.as_str() {
        // Images
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "tiff" | "tif"
        | "heic" | "heif" | "raw" | "cr2" | "nef" | "arw" | "ico" | "avif" => Some(pics.clone()),

        // Videos
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "3gp"
        | "vob" | "ogv" | "mpg" | "mpeg" => Some(vids.clone()),

        // Audio
        "mp3" | "flac" | "wav" | "ogg" | "aac" | "m4a" | "opus" | "wma" | "aiff"
        | "mid" | "midi" | "amr" => Some(music.clone()),

        // Documents — text
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "txt" | "md" | "rst" | "tex"
        | "pages" | "wpd" => Some(docs.clone()),

        // Documents — data/config
        "ron" | "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" | "conf" => {
            Some(docs.join("Config"))
        }

        // Spreadsheets
        "xls" | "xlsx" | "ods" | "csv" | "tsv" | "numbers" => Some(docs.join("Spreadsheets")),

        // Presentations
        "ppt" | "pptx" | "odp" | "key" => Some(docs.join("Presentations")),

        // Ebooks
        "epub" | "mobi" | "azw" | "azw3" | "fb2" | "djvu" | "cbz" | "cbr" => {
            Some(docs.join("Books"))
        }

        // Code / scripts
        "rs" | "py" | "js" | "ts" | "sh" | "bash" | "zsh" | "fish" | "rb" | "go"
        | "c" | "cpp" | "h" | "hpp" | "java" | "kt" | "swift" | "lua" | "php"
        | "html" | "css" | "scss" | "sql" => Some(home.join("Code")),

        // AppImages
        "appimage" => Some(home.join("Applications")),

        // Disk images / ISOs
        "iso" | "img" | "dmg" => Some(home.join("Images")),

        // Installers / packages
        "deb" | "rpm" | "pkg" | "exe" | "msi" => Some(downloads.clone()),

        // Archives — stay in Downloads
        "zip" | "tar" | "gz" | "xz" | "bz2" | "7z" | "rar" | "zst" | "lz4" => {
            return None;
        }

        // Fonts
        "ttf" | "otf" | "woff" | "woff2" | "eot" => Some(home.join(".fonts")),

        _ => None,
    };
    // Name-based overrides
    let dest = dest.or_else(|| {
        if name.contains("resume") || name.contains("curriculum") || (name.contains("cv") && !name.ends_with(".csv")) {
            Some(docs.join("Career"))
        } else if name.contains("invoice") || name.contains("receipt") || name.contains("tax") || name.contains("billing") {
            Some(docs.join("Finance"))
        } else if name.contains("screenshot") || name.starts_with("screen") || name.starts_with("scrot") {
            Some(pics.join("Screenshots"))
        } else if name.contains("wallpaper") || name.contains("background") {
            Some(pics.join("Wallpapers"))
        } else if name.contains("backup") || name.ends_with(".bak") {
            Some(home.join("Backups"))
        } else {
            None
        }
    })?;
    // Don't suggest if already in the right place
    if path.parent() == Some(dest.as_path()) {
        return None;
    }
    let dest_file = dest.join(path.file_name()?);
    let reason = format!(
        "{} looks like it belongs in {}",
        path.file_name()?.to_str()?,
        dest.display()
    );

    Some(MoveSuggestion { from: path.to_path_buf(), to: dest_file, reason })
}
