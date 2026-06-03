// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/vault/hidden_apps.rs
// Hidden apps — moves a user .desktop file into the vault (encrypted) so the
// app disappears from every system menu, and shows it as a launchable grid in
// the vault when unlocked. Launch runs the stored exec directly (no restore
// needed). Unhide writes the .desktop back to its original location.
//
// Scope: user apps only (~/.local/share/applications). Encrypted with the vault
// key, so the grid is only visible and launchable when the vault is unlocked.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::encryption;

/// Where encrypted hidden-app blobs + metadata live.
pub fn hidden_dir() -> PathBuf {
    super::vault_dir().join("hidden_apps")
}

const HIDDEN_ENC_EXT: &str = ".enc";
const HIDDEN_META_EXT: &str = ".meta";

/// Metadata stored per hidden app (the .meta sidecar).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenAppMeta {
    /// Absolute path the .desktop file came from, for restore on unhide.
    pub original_path: String,
    /// Display name for the grid.
    pub name: String,
    /// Icon name or absolute path for the grid.
    pub icon: String,
    /// Exec command to launch the app directly from the vault.
    pub exec: String,
    /// When it was hidden (unix seconds).
    pub hidden_at: u64,
}

/// What the UI sees.
#[derive(Debug, Clone)]
pub struct HiddenApp {
    /// Random ID — the encrypted blob filename without extension.
    pub id: String,
    pub meta: HiddenAppMeta,
}

/// Hide a user .desktop file: encrypt its contents into the vault and delete
/// the original so the app disappears from all system menus. Returns the
/// HiddenApp record for the in-memory list.
pub fn hide(key: &[u8], desktop_path: &std::path::Path) -> Result<HiddenApp, String> {
    let contents = std::fs::read_to_string(desktop_path)
        .map_err(|e| format!("Could not read .desktop: {e}"))?;

    let name = field(&contents, "Name").unwrap_or_else(|| "Hidden App".to_string());
    let icon = field(&contents, "Icon").unwrap_or_default();
    let exec_raw = field(&contents, "Exec").unwrap_or_default();
    let exec = crate::utils::strip_desktop_placeholders(&exec_raw);

    let encrypted = encryption::encrypt_data(key, contents.as_bytes())?;
    let id = encryption::random_id();

    let dir = hidden_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Could not create hidden dir: {e}"))?;
    std::fs::write(dir.join(format!("{id}{HIDDEN_ENC_EXT}")), &encrypted)
        .map_err(|e| format!("Could not write encrypted app: {e}"))?;

    let meta = HiddenAppMeta {
        original_path: desktop_path.display().to_string(),
        name,
        icon,
        exec,
        hidden_at: encryption::unix_now(),
    };
    let meta_json = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("Could not serialize metadata: {e}"))?;
    std::fs::write(dir.join(format!("{id}{HIDDEN_META_EXT}")), meta_json)
        .map_err(|e| format!("Could not write metadata: {e}"))?;

    std::fs::remove_file(desktop_path)
        .map_err(|e| format!("Could not remove original .desktop: {e}"))?;

    Ok(HiddenApp { id, meta })
}

/// Extract a top-level `Key=value` field from .desktop contents.
fn field(contents: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    contents
        .lines()
        .find(|l| l.starts_with(&prefix))
        .map(|l| l[prefix.len()..].trim().to_string())
}
