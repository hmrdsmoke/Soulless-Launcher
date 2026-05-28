// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use secrecy::{ExposeSecret, SecretVec};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

// ── Constants ─────────────────────────────────────────────────────────────────

const KEY_FILE: &str = ".key";
const META_EXT: &str = ".meta";
const ENC_EXT: &str = ".enc";
const NONCE_SIZE: usize = 12;

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

        match hash_password(&self.password_input) {
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

        match derive_key(&self.password_input) {
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

        match verify_password(&self.password_input, stored.trim()) {
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

        match derive_key(&self.password_input) {
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

        let mime_type = mime_from_path(source);

        let encrypted = encrypt_data(key, &plaintext)?;

        let id = random_id();
        let dir = vault_dir();
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Could not create vault dir: {e}"))?;

        fs::write(dir.join(format!("{id}{ENC_EXT}")), &encrypted)
            .map_err(|e| format!("Could not write encrypted file: {e}"))?;

        let meta = VaultFileMeta {
            original_name: original_name.clone(),
            mime_type,
            size,
            added_at: unix_now(),
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

        let plaintext = decrypt_data(key, &encrypted)?;

        let entry = self
            .entries
            .iter()
            .find(|e| e.id == entry_id)
            .ok_or_else(|| "Entry not found".to_string())?;

        let tmp_dir = std::env::temp_dir()
            .join(format!("soulless-vault-{}", random_id()));

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

// ── Crypto helpers ────────────────────────────────────────────────────────────

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed = PasswordHash::new(hash).map_err(|e| e.to_string())?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Derives a 32-byte encryption key from the password using Argon2.
/// Uses a fixed salt so the key is stable across sessions for the same password.
fn derive_key(password: &str) -> Result<SecretVec<u8>, String> {
    let mut key = vec![0u8; 32];
    Argon2::default()
        .hash_password_into(
            password.as_bytes(),
            b"soulless-vault-kdf-salt-v1",
            &mut key,
        )
        .map_err(|e| e.to_string())?;
    Ok(SecretVec::new(key))
}

/// Encrypts plaintext with ChaCha20-Poly1305.
/// Output format: [12-byte nonce][ciphertext]
fn encrypt_data(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption error: {e}"))?;

    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypts data encrypted by `encrypt_data`.
fn decrypt_data(key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < NONCE_SIZE {
        return Err("Encrypted data is too short.".into());
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed — wrong password or corrupted file.".into())
}

fn random_id() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Extension-based MIME type detection — no external crate needed.
fn mime_from_path(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        // Video
        "mp4" | "m4v" => "video/mp4",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "flv" => "video/x-flv",
        "wmv" => "video/x-ms-wmv",
        // Image
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "tiff" | "tif" => "image/tiff",
        // Audio
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",
        // Documents
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "html" | "htm" => "text/html",
        // Archives
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "xz" => "application/x-xz",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",
        // Default
        _ => "application/octet-stream",
    }
    .to_string()
}

// === DONE ===
// Added drag_hover: bool field to Vault struct :: done
// Initialised drag_hover: false in Vault::new() :: done
// All crypto, file IO, and lock/unlock logic unchanged :: done

// === DONE ===
// Replaced mime_guess dependency with built-in mime_from_path() :: done
// Covers video, image, audio, documents, archives :: done
// Falls back to application/octet-stream for unknown types :: done
// All crypto logic preserved unchanged :: done
// Vault::new() is the correct constructor — Vault::load() removed :: done
// finish_setup() is the correct setup method :: done
// remove_file() is the correct removal method :: done
// === DONE ===
// Full vault backend implemented :: done
// Argon2 password hashing :: done
// Argon2 key derivation (stable per password) :: done
// ChaCha20-Poly1305 per-file encryption :: done
// Random nonce per file :: done
// Metadata stored alongside each encrypted blob :: done
// add_file: encrypt + move original, pkexec fallback for permission errors :: done
// open_file: decrypt to temp, xdg-open, cleanup on drop :: done
// remove_file: delete enc + meta :: done
// load_entries: scan vault dir on unlock :: done
// Vault locks and zeroizes key on drop :: done
// temp files cleaned up on launcher close via Drop :: done

// === DONE ===
// Fixed: bare `iced` imports replaced with `cosmic::iced` :: done
// Fixed: iced::alignment::Horizontal → cosmic::iced::alignment::Horizontal :: done
// Fixed: iced::border::rounded → cosmic::iced::border::rounded :: done
// Vault is placeholder — encryption logic to be added :: MRV
// Ready for future integration with age or other crypto :: MRV
// === YOUR ORIGINAL COMMENTS (preserved exactly) ===
// Vault is placeholder for now - encryption logic to be added :: MRV
// Ready for future integration with age or other crypto :: MRV
// === DONE ===
// Basic vault UI structure ready for expansion
pub mod ui;
