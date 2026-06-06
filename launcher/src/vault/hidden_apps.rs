// GPL-3.0-or-later - see LICENSE file for full terms
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
    let meta_json = serde_json::to_string(&meta)
        .map_err(|e| format!("Could not serialize metadata: {e}"))?;
    let meta_enc = encryption::encrypt_data(key, meta_json.as_bytes())?;
    std::fs::write(dir.join(format!("{id}{HIDDEN_META_EXT}")), meta_enc)
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

/// Decrypt and load all hidden-app metadata for the grid. Requires the key
/// (metadata is encrypted), so this only works when the vault is unlocked.
pub fn load_all(key: &[u8]) -> Vec<HiddenApp> {
    let mut out = Vec::new();
    let dir = hidden_dir();
    let Ok(read_dir) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("meta") {
            continue;
        }
        let Ok(enc) = std::fs::read(&path) else { continue };
        let Ok(plain) = encryption::decrypt_data(key, &enc) else { continue };
        let Ok(meta) = serde_json::from_slice::<HiddenAppMeta>(&plain) else { continue };
        let id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        // Only include if the encrypted .desktop blob still exists
        if dir.join(format!("{id}{HIDDEN_ENC_EXT}")).exists() {
            out.push(HiddenApp { id, meta });
        }
    }
    out
}

/// Restore a hidden app: decrypt the stored .desktop, write it back to its
/// original path, and delete the encrypted blob + metadata.
pub fn unhide(key: &[u8], app: &HiddenApp) -> Result<(), String> {
    let dir = hidden_dir();
    let enc_path = dir.join(format!("{}{}", app.id, HIDDEN_ENC_EXT));
    let enc = std::fs::read(&enc_path)
        .map_err(|e| format!("Could not read hidden app: {e}"))?;
    let contents = encryption::decrypt_data(key, &enc)?;
    std::fs::write(&app.meta.original_path, &contents)
        .map_err(|e| format!("Could not restore .desktop: {e}"))?;
    let _ = std::fs::remove_file(&enc_path);
    let _ = std::fs::remove_file(dir.join(format!("{}{}", app.id, HIDDEN_META_EXT)));
    Ok(())
}

