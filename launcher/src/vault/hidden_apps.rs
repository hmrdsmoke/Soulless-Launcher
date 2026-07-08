// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/vault/hidden_apps.rs
// Hidden apps — hides an app from the launcher into the vault, for ANY source.
//
// Two modes, decided by whether the app has a .desktop file:
//   - Desktop apps (original_path = Some): the .desktop is encrypted into the
//     vault and the original deleted, so the app disappears from every system
//     menu. Unhide writes it back.
//   - Sourced apps (Steam, Flatpak-manifest, etc.; original_path = None): there
//     is no file we own to delete — the app lives in a third-party manifest.
//     These are hidden by SOURCE ID via the filter below.
//
// EVERY hidden app is also recorded in the filter (see below), so a hidden app
// stays hidden even if its source reappears (e.g. Steam re-creating a shortcut).
//
// ── The filter ────────────────────────────────────────────────────────────────
// The search index is built at startup, BEFORE the vault is unlocked, so the
// launcher must know which source IDs to drop WITHOUT the vault key. The filter
// (hidden_apps/.filter) holds one Blake2b hash per hidden app:
//     Blake2b512( "soulless-hidden-v1" || per-install salt || source_id )
// Hashes — not plaintext IDs — because the .meta names are encrypted precisely
// so hidden apps aren't identifiable while locked; a plaintext "steam:238960"
// line would leak the same information the encryption protects. The per-install
// salt (the vault's .salt, readable pre-unlock by design — it's a KDF salt)
// prevents cross-install precomputation. Launch runs the stored exec directly
// from the unlocked grid; no restore needed.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use blake2::{Blake2b512, Digest};

use super::encryption;

/// Where encrypted hidden-app blobs + metadata + the filter live.
pub fn hidden_dir() -> PathBuf {
    super::vault_dir().join("hidden_apps")
}

const HIDDEN_ENC_EXT: &str = ".enc";
const HIDDEN_META_EXT: &str = ".meta";
const FILTER_FILE: &str = ".filter";
const HASH_DOMAIN: &[u8] = b"soulless-hidden-v1";

/// Metadata stored per hidden app (the .meta sidecar, encrypted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenAppMeta {
    /// Absolute path the .desktop file came from, for restore on unhide.
    /// None for sourced apps (Steam etc.) — nothing on disk to restore.
    /// (Old metas stored a plain String; serde reads that as Some.)
    pub original_path: Option<String>,
    /// Indexer source ID (e.g. "steam:238960", "desktop:foo.desktop") — the key
    /// used in the filter. Old metas predate this field, hence the default:
    /// they were never in the filter, so unhide has nothing to remove.
    #[serde(default)]
    pub source_id: Option<String>,
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

/// Hide an app of ANY source. Caller (Vault front door) passes the indexed
/// fields; `desktop_path` is Some only when the app has a .desktop we own.
///
/// Order matters: encrypted blob + meta + filter are all written BEFORE the
/// original .desktop is deleted, so a failure at any step never strands the
/// app in a half-hidden state that can't be recovered from the vault grid.
pub fn hide(
    key: &[u8],
    source_id: &str,
    name: &str,
    icon: &str,
    exec: &str,
    desktop_path: Option<&Path>,
) -> Result<HiddenApp, String> {
    let id = encryption::random_id();
    let dir = hidden_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Could not create hidden dir: {e}"))?;

    // Desktop apps: encrypt the .desktop contents for restore-on-unhide.
    if let Some(dp) = desktop_path {
        let contents = std::fs::read_to_string(dp)
            .map_err(|e| format!("Could not read .desktop: {e}"))?;
        let encrypted = encryption::encrypt_data(key, contents.as_bytes())?;
        std::fs::write(dir.join(format!("{id}{HIDDEN_ENC_EXT}")), &encrypted)
            .map_err(|e| format!("Could not write encrypted app: {e}"))?;
    }

    let meta = HiddenAppMeta {
        original_path: desktop_path.map(|p| p.display().to_string()),
        source_id: Some(source_id.to_string()),
        name: name.to_string(),
        icon: icon.to_string(),
        exec: exec.to_string(),
        hidden_at: encryption::unix_now(),
    };
    let meta_json = serde_json::to_string(&meta)
        .map_err(|e| format!("Could not serialize metadata: {e}"))?;
    let meta_enc = encryption::encrypt_data(key, meta_json.as_bytes())?;
    std::fs::write(dir.join(format!("{id}{HIDDEN_META_EXT}")), meta_enc)
        .map_err(|e| format!("Could not write metadata: {e}"))?;

    // Record in the filter so the app stays out of the search index across
    // restarts, locked or unlocked. On failure, roll back the blob + meta so
    // nothing is left hidden-but-unfiltered.
    if let Err(e) = filter_add(source_id) {
        let _ = std::fs::remove_file(dir.join(format!("{id}{HIDDEN_ENC_EXT}")));
        let _ = std::fs::remove_file(dir.join(format!("{id}{HIDDEN_META_EXT}")));
        return Err(e);
    }

    // Last step: delete the original .desktop (desktop apps only). If this
    // fails the app is still hidden — the filter drops it from the index —
    // and unhide cleans up normally.
    if let Some(dp) = desktop_path {
        std::fs::remove_file(dp)
            .map_err(|e| format!("Could not remove original .desktop: {e}"))?;
    }

    Ok(HiddenApp { id, meta })
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
        // Desktop-backed entries must still have their encrypted .desktop blob;
        // sourced entries (Steam etc.) never had one.
        if meta.original_path.is_some()
            && !dir.join(format!("{id}{HIDDEN_ENC_EXT}")).exists()
        {
            continue;
        }
        out.push(HiddenApp { id, meta });
    }
    out
}

/// Restore a hidden app. Desktop apps: decrypt the stored .desktop and write it
/// back. All apps: remove the filter entry so the source reappears in search,
/// then delete the blob + metadata.
pub fn unhide(key: &[u8], app: &HiddenApp) -> Result<(), String> {
    let dir = hidden_dir();
    let enc_path = dir.join(format!("{}{}", app.id, HIDDEN_ENC_EXT));

    if let Some(original_path) = &app.meta.original_path {
        let enc = std::fs::read(&enc_path)
            .map_err(|e| format!("Could not read hidden app: {e}"))?;
        let contents = encryption::decrypt_data(key, &enc)?;
        std::fs::write(original_path, &contents)
            .map_err(|e| format!("Could not restore .desktop: {e}"))?;
    }

    if let Some(source_id) = &app.meta.source_id {
        filter_remove(source_id)?;
    }

    let _ = std::fs::remove_file(&enc_path);
    let _ = std::fs::remove_file(dir.join(format!("{}{}", app.id, HIDDEN_META_EXT)));
    Ok(())
}

// ── Hidden-source filter ──────────────────────────────────────────────────────

/// Loaded set of hidden source-ID hashes. Built once per index rebuild; does
/// NOT require the vault to be unlocked (reads only .salt + .filter).
pub struct HiddenFilter {
    salt: Option<Vec<u8>>,
    hashes: HashSet<String>,
}

impl HiddenFilter {
    /// Load the filter from disk. Missing vault / salt / filter file all yield
    /// an empty filter (nothing hidden).
    pub fn load() -> Self {
        let salt = super::read_salt();
        let hashes = match (&salt, std::fs::read_to_string(filter_path())) {
            (Some(_), Ok(text)) => text
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect(),
            _ => HashSet::new(),
        };
        Self { salt, hashes }
    }

    /// True if this source ID is hidden. Cheap: one Blake2b of a short string.
    pub fn is_hidden(&self, source_id: &str) -> bool {
        if self.hashes.is_empty() {
            return false;
        }
        let Some(salt) = &self.salt else { return false };
        self.hashes.contains(&hash_id(salt, source_id))
    }
}

fn filter_path() -> PathBuf {
    hidden_dir().join(FILTER_FILE)
}

/// Blake2b512( domain || salt || source_id ) as lowercase hex.
fn hash_id(salt: &[u8], source_id: &str) -> String {
    let mut hasher = Blake2b512::new();
    hasher.update(HASH_DOMAIN);
    hasher.update(salt);
    hasher.update(source_id.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Add a source ID's hash to the filter file (idempotent).
fn filter_add(source_id: &str) -> Result<(), String> {
    let salt = super::read_salt()
        .ok_or_else(|| "Vault salt missing — cannot record hidden app.".to_string())?;
    let mut hashes: HashSet<String> = std::fs::read_to_string(filter_path())
        .map(|t| {
            t.lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default();
    hashes.insert(hash_id(&salt, source_id));
    write_filter(&hashes)
}

/// Remove a source ID's hash from the filter file. Deletes the file when the
/// last entry goes, keeping the vault dir clean.
fn filter_remove(source_id: &str) -> Result<(), String> {
    let Some(salt) = super::read_salt() else {
        return Ok(()); // no salt → nothing was ever filtered
    };
    let Ok(text) = std::fs::read_to_string(filter_path()) else {
        return Ok(()); // no filter file → nothing to remove
    };
    let target = hash_id(&salt, source_id);
    let hashes: HashSet<String> = text
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && *l != target)
        .collect();
    if hashes.is_empty() {
        let _ = std::fs::remove_file(filter_path());
        return Ok(());
    }
    write_filter(&hashes)
}

fn write_filter(hashes: &HashSet<String>) -> Result<(), String> {
    let mut lines: Vec<&str> = hashes.iter().map(|s| s.as_str()).collect();
    lines.sort_unstable(); // stable file contents → clean diffs if ever inspected
    let body = lines.join("\n") + "\n";
    std::fs::create_dir_all(hidden_dir())
        .map_err(|e| format!("Could not create hidden dir: {e}"))?;
    std::fs::write(filter_path(), body)
        .map_err(|e| format!("Could not write hidden filter: {e}"))
}