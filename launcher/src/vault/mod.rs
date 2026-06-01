pub mod ui;
pub mod encryption;
// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use secrecy::{ExposeSecret, SecretVec};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

// ── Constants ─────────────────────────────────────────────────────────────────

const KEY_FILE: &str = ".key";
const META_EXT: &str = ".meta";
const ENC_EXT: &str = ".enc";

// ── Vault directory ───────────────────────────────────────────────────────────

pub fn vault_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".local/share/soulless/vault")
}

// ── File metadata ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultFileMeta {
    pub original_name: String,
    pub mime_type: String,
    pub size: u64,
    pub added_at: u64,
}

// ── Vault entry — what the UI sees ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VaultEntry {
    /// Random ID — the encrypted blob filename without extension
    pub id: String,
    pub meta: VaultFileMeta,
}

// ── Lock state ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultLockState {
    /// First launch — no password set yet
    Uninitialized,
    /// Password set but not entered this session
    Locked,
    /// Password verified this session
    Unlocked,
}

// ── Vault ─────────────────────────────────────────────────────────────────────

pub struct Vault {
    pub lock_state: VaultLockState,

    /// Password field (cleared immediately after use)
    pub password_input: String,
    /// Confirm field — only used during setup
    pub confirm_input: String,
    /// True when we are in setup mode (first launch confirm step)
    pub is_setup: bool,

    /// Derived encryption key — only present when unlocked
    derived_key: Option<SecretVec<u8>>,

    /// File list — only populated when unlocked
    pub entries: Vec<VaultEntry>,

    /// Temp dirs opened this session — all cleaned up on launcher close
    pub temp_dirs: Vec<PathBuf>,

    pub error: Option<String>,
    pub status: Option<String>,

    /// True while a drag is hovering over the vault drop zone
    pub drag_hover: bool,
    /// ID of the file whose context menu is open (None = closed)
    pub context_menu_entry: Option<String>,
    
   
    
}


impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

impl Vault {
    pub fn new() -> Self {
        let key_path = vault_dir().join(KEY_FILE);
        let lock_state = if key_path.exists() {
            VaultLockState::Locked
        } else {
            VaultLockState::Uninitialized
        };

        Self {
            lock_state,
            password_input: String::new(),
            confirm_input: String::new(),
            is_setup: false,
            derived_key: None,
            entries: Vec::new(),
            temp_dirs: Vec::new(),
            error: None,
            status: None,
            drag_hover: false,
            context_menu_entry: None,
        }
    }

    // ── Setup (first launch) ──────────────────────────────────────────────

    #[allow(dead_code)] // issue #3
    pub fn begin_setup(&mut self) {
        self.is_setup = true;
        self.error = None;
    }

    pub fn finish_setup(&mut self) -> bool {
        if self.password_input.is_empty() {
            self.error = Some("Password cannot be empty.".into());
            return false;
        }
        if self.password_input.len() < 8 {
            self.error = Some("Password must be at least 8 characters.".into());
            return false;
        }
        if self.password_input != self.confirm_input {
            self.error = Some("Passwords do not match.".into());
            return false;
        }

        let dir = vault_dir();
        if let Err(e) = fs::create_dir_all(&dir) {
            self.error = Some(format!("Could not create vault directory: {e}"));
            return false;
        }

        match encryption::hash_password(&self.password_input) {
            Ok(hash) => {
                if let Err(e) = fs::write(dir.join(KEY_FILE), &hash) {
                    self.error = Some(format!("Could not save key file: {e}"));
                    return false;
                }
            }
            Err(e) => {
                self.error = Some(format!("Password hashing failed: {e}"));
                return false;
            }
        }

        match encryption::derive_key(&self.password_input) {
            Ok(key) => {
                self.derived_key = Some(key);
                self.lock_state = VaultLockState::Unlocked;
                self.is_setup = false;
                self.error = None;
                self.status = Some("Vault created. Welcome.".into());
                self.wipe_inputs();
                true
            }
            Err(e) => {
                self.error = Some(format!("Key derivation failed: {e}"));
                false
            }
        }
    }

    // ── Unlock ────────────────────────────────────────────────────────────

    pub fn unlock(&mut self) -> bool {
        let key_path = vault_dir().join(KEY_FILE);

        let stored = match fs::read_to_string(&key_path) {
            Ok(s) => s,
            Err(e) => {
                self.error = Some(format!("Could not read key file: {e}"));
                return false;
            }
        };

        match encryption::verify_password(&self.password_input, stored.trim()) {
            Ok(true) => {}
            Ok(false) => {
                self.error = Some("Incorrect password.".into());
                self.password_input.clear();
                return false;
            }
            Err(e) => {
                self.error = Some(format!("Verification error: {e}"));
                return false;
            }
        }

        match encryption::derive_key(&self.password_input) {
            Ok(key) => {
                self.derived_key = Some(key);
                self.lock_state = VaultLockState::Unlocked;
                self.error = None;
                self.wipe_inputs();
                self.load_entries();
                true
            }
            Err(e) => {
                self.error = Some(format!("Key derivation failed: {e}"));
                false
            }
        }
    }

    // ── Lock ──────────────────────────────────────────────────────────────

    pub fn lock(&mut self) {
        self.derived_key = None;
        self.entries.clear();
        self.lock_state = VaultLockState::Locked;
        self.error = None;
        self.status = None;
        self.wipe_inputs();
    }

    // ── Add file ──────────────────────────────────────────────────────────

    /// Encrypts `source` into the vault then removes the original.
    /// Returns Err(message) if anything goes wrong.
    /// If the remove fails due to permissions, returns a special
    /// Err starting with "NEEDS_PKEXEC:" so the caller can retry.
    pub fn add_file(&mut self, source: &Path) -> Result<(), String> {
        let key = self.key_bytes()?;

        let original_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid filename".to_string())?
            .to_string();

        let plaintext = fs::read(source)
            .map_err(|e| format!("Could not read file: {e}"))?;

        let size = plaintext.len() as u64;

        let mime_type = encryption::mime_from_path(source);

        let encrypted = encryption::encrypt_data(key, &plaintext)?;

        let id = encryption::random_id();
        let dir = vault_dir();
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Could not create vault dir: {e}"))?;

        fs::write(dir.join(format!("{id}{ENC_EXT}")), &encrypted)
            .map_err(|e| format!("Could not write encrypted file: {e}"))?;

        let meta = VaultFileMeta {
            original_name: original_name.clone(),
            mime_type,
            size,
            added_at: encryption::unix_now(),
        };

        let meta_json = serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Could not serialize metadata: {e}"))?;

        fs::write(dir.join(format!("{id}{META_EXT}")), meta_json)
            .map_err(|e| format!("Could not write metadata: {e}"))?;

        // Try to remove the original
        if let Err(e) = fs::remove_file(source) {
            eprintln!(
                "Could not remove original ({}): {e} — needs pkexec",
                source.display()
            );
            // Encrypted copy is safe — just the original removal failed
            self.entries.push(VaultEntry { id, meta });
            return Err(format!("NEEDS_PKEXEC:{}", source.display()));
        }

        self.entries.push(VaultEntry { id, meta });
        self.status = Some(format!("'{original_name}' added to vault."));
        Ok(())
    }

    #[allow(dead_code)] // issue #3
    /// Retry the original file removal using pkexec.
    pub fn remove_original_pkexec(source: &Path) -> Result<(), String> {
        let status = std::process::Command::new("pkexec")
            .arg("rm")
            .arg("--")
            .arg(source)
            .status()
            .map_err(|e| format!("Could not spawn pkexec: {e}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "pkexec rm failed with code {:?}",
                status.code()
            ))
        }
    }

    // ── Open file ─────────────────────────────────────────────────────────

    pub fn open_file(&mut self, entry_id: &str) -> Result<(), String> {
        let key = self.key_bytes()?;

        let dir = vault_dir();
        let enc_path = dir.join(format!("{entry_id}{ENC_EXT}"));

        let encrypted = fs::read(&enc_path)
            .map_err(|e| format!("Could not read encrypted file: {e}"))?;

        let plaintext = encryption::decrypt_data(key, &encrypted)?;

        let entry = self
            .entries
            .iter()
            .find(|e| e.id == entry_id)
            .ok_or_else(|| "Entry not found".to_string())?;

        let tmp_dir = std::env::temp_dir()
            .join(format!("soulless-vault-{}", encryption::random_id()));

        fs::create_dir_all(&tmp_dir)
            .map_err(|e| format!("Could not create temp dir: {e}"))?;

        let tmp_file = tmp_dir.join(&entry.meta.original_name);

        fs::write(&tmp_file, &plaintext)
            .map_err(|e| format!("Could not write temp file: {e}"))?;

        self.temp_dirs.push(tmp_dir);

        std::process::Command::new("xdg-open")
            .arg(&tmp_file)
            .spawn()
            .map_err(|e| format!("Could not open file: {e}"))?;

        Ok(())
    }


    // ── Export file from vault to Downloads ───────────────────────────────
    pub fn export_file(&mut self, entry_id: &str) -> Result<(), String> {
        let key = self.key_bytes()?;
        let dir = vault_dir();
        let enc_path = dir.join(format!("{entry_id}{ENC_EXT}"));
        let encrypted = fs::read(&enc_path)
            .map_err(|e| format!("Could not read encrypted file: {e}"))?;
        let plaintext = encryption::decrypt_data(key, &encrypted)?;
        let entry = self
            .entries
            .iter()
            .find(|e| e.id == entry_id)
            .ok_or_else(|| "Entry not found".to_string())?;
        let downloads = dirs::download_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/Downloads"));
        let dest = downloads.join(&entry.meta.original_name);
        fs::write(&dest, &plaintext)
            .map_err(|e| format!("Could not export file: {e}"))?;
        Ok(())
    }
    // ── Remove file from vault ────────────────────────────────────────────

    pub fn remove_file(&mut self, entry_id: &str) -> Result<(), String> {
        let dir = vault_dir();

        fs::remove_file(dir.join(format!("{entry_id}{ENC_EXT}")))
            .map_err(|e| format!("Could not remove encrypted file: {e}"))?;

        let _ = fs::remove_file(dir.join(format!("{entry_id}{META_EXT}")));

        self.entries.retain(|e| e.id != entry_id);
        self.status = Some("File removed from vault.".into());
        Ok(())
    }

    // ── Cleanup temp files (call on launcher close) ───────────────────────

    pub fn cleanup_temp(&self) {
        for dir in &self.temp_dirs {
            let _ = fs::remove_dir_all(dir);
        }
    }

    // ── Load entries from disk ────────────────────────────────────────────

    fn load_entries(&mut self) {
        self.entries.clear();

        let dir = vault_dir();
        let Ok(read_dir) = fs::read_dir(&dir) else {
            return;
        };

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("meta") {
                continue;
            }

            let Ok(json) = fs::read_to_string(&path) else {
                continue;
            };

            let Ok(meta) = serde_json::from_str::<VaultFileMeta>(&json)
            else {
                continue;
            };

            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            // Only include if the encrypted blob actually exists
            let enc_path = dir.join(format!("{id}{ENC_EXT}"));
            if !enc_path.exists() {
                continue;
            }

            self.entries.push(VaultEntry { id, meta });
        }

        // Sort by date added, newest first
        self.entries.sort_by(|a, b| {
            b.meta.added_at.cmp(&a.meta.added_at)
        });
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    fn key_bytes(&self) -> Result<&[u8], String> {
        self.derived_key
            .as_ref()
            .map(|k| k.expose_secret().as_slice())
            .ok_or_else(|| "Vault is locked.".to_string())
    }

    fn wipe_inputs(&mut self) {
        self.password_input.zeroize();
        self.confirm_input.zeroize();
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        self.cleanup_temp();
        self.wipe_inputs();
    }
}
