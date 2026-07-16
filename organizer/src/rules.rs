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
    let ext = path.extension()?.to_str()?.to_lowercase();

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

        // Documents — structured data (and config-flavored files). Named
        // "Data", not "Config": a downloaded .json is usually an export or a
        // dump, almost never configuration, and "Config" in Documents reads
        // confusingly against ~/.config. sql lives here too — a dump is data.
        "ron" | "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" | "conf"
        | "sql" => Some(docs.join("Data")),

        // Spreadsheets
        "xls" | "xlsx" | "ods" | "csv" | "tsv" | "numbers" => Some(docs.join("Spreadsheets")),

        // Presentations
        "ppt" | "pptx" | "odp" | "key" => Some(docs.join("Presentations")),

        // Ebooks
        "epub" | "mobi" | "azw" | "azw3" | "fb2" | "djvu" | "cbz" | "cbr" => {
            Some(docs.join("Books"))
        }

        // Code — real source files only. Pruned from the old list: shell
        // scripts (a downloaded .sh is almost always a run-once installer),
        // html/css/scss (saved webpages, not projects), sql (moved to Data).
        "rs" | "py" | "js" | "ts" | "rb" | "go"
        | "c" | "cpp" | "h" | "hpp" | "java" | "kt" | "swift" | "lua" | "php" => {
            Some(home.join("Code"))
        }

        // Run-once installers and page assets — stay where they landed.
        "sh" | "bash" | "zsh" | "fish" | "html" | "css" | "scss" => return None,

        // AppImages
        "appimage" => Some(home.join("Applications")),

        // Disk images: mount/burn-once artifacts — stay put. (The old
        // ~/Images destination also collided with localized XDG Pictures,
        // which literally IS ~/Images on French-locale systems.)
        "iso" | "img" | "dmg" => return None,

        // Installers: install-and-delete artifacts — stay put. (Previously
        // Some(downloads), which no-op'd for files already in Downloads but
        // would usher a .deb found on the Desktop into Downloads.)
        "deb" | "rpm" | "pkg" | "exe" | "msi" => return None,

        // Archives — stay in Downloads
        "zip" | "tar" | "gz" | "xz" | "bz2" | "7z" | "rar" | "zst" | "lz4" => {
            return None;
        }

        // Fonts: moving into the fonts dir effectively installs them — which
        // is what a font download wants. Modern fontconfig path, not the
        // deprecated ~/.fonts. woff/woff2/eot are web-page assets nobody
        // installs locally — stay put.
        "ttf" | "otf" => {
            Some(dirs::font_dir().unwrap_or_else(|| home.join(".local/share/fonts")))
        }
        "woff" | "woff2" | "eot" => return None,

        _ => None,
    };
    // Extensions only, deliberately: an extension is a fact about the file;
    // a filename is a claim by whoever named it ("syntax.pdf" contains "tax",
    // "opencv_notes.pdf" contains "cv"). The organizer acts on facts and
    // leaves claims to the human — that's what Skip is for.
    let dest = dest?;
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
