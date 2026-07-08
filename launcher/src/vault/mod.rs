// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/vault/mod.rs
// Encrypted vault core: lock state, storage, and key handling.

pub mod ui;
pub mod encryption;
pub mod hidden_apps;

use secrecy::{ExposeSecret, SecretVec};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

// ── Constants ─────────────────────────────────────────────────────────────────

const KEY_FILE: &str = ".key";
const SALT_FILE: &str = ".salt"; // per-install v2 key-derivation salt
const VERSION_FILE: &str = ".version"; // on-disk format version stamp
const META_EXT: &str = ".meta";
const ENC_EXT: &str = ".enc";

// ── Vault directory ───────────────────────────────────────────────────────────

pub fn vault_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".local/share/soulless/vault")
}

/// Where decrypted files are briefly materialized when opened. RAM-backed tmpfs
/// (XDG_RUNTIME_DIR is /run/user/$UID, never touches persistent disk and is wiped
/// on logout/poweroff). Falls back to /dev/shm, then — last resort — temp_dir().
/// We deliberately avoid /tmp when we can: on tmpfs the plaintext never lands on
/// the physical disk at all.
fn runtime_tmp_base() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(x);
        if p.is_dir() {
            return p;
        }
    }
    let shm = PathBuf::from("/dev/shm");
    if shm.is_dir() {
        return shm;
    }
    std::env::temp_dir()
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
    /// An OLD-FORMAT vault was detected. The user must back up + empty it, then
    /// confirm the upgrade (which wipes and recreates the vault in v2 format).
    /// We never silently migrate ciphertext — empty-and-rebuild keeps it simple
    /// and unbrickable.
    NeedsUpgrade,
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

    /// Temp dirs opened this session — all cleaned up on launcher close.
    /// Each carries the child process so we can wipe as soon as it exits.
    pub temp_dirs: Vec<PathBuf>,
    /// Child processes launched for opened files, paired with their temp dir,
    /// so cleanup can happen the moment the viewer/editor exits.
    open_procs: Vec<(std::process::Child, PathBuf)>,

    pub error: Option<String>,
    pub status: Option<String>,

    /// True while a drag is hovering over the vault drop zone
    pub drag_hover: bool,
    /// ID of the file whose context menu is open (None = closed)
    pub context_menu_entry: Option<String>,
    /// Hidden apps — populated when unlocked (encrypted .desktop files).
    pub hidden_apps: Vec<hidden_apps::HiddenApp>,
    /// ID of the hidden app whose context menu is open (None = closed)
    pub hidden_context_menu: Option<String>,
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

impl Vault {
    pub fn new() -> Self {
        let dir = vault_dir();
        let key_path = dir.join(KEY_FILE);

        // Detect the on-disk state:
        //   no .key            -> first run, needs setup
        //   .key, old format   -> needs the empty-and-rebuild upgrade
        //   .key, current      -> normal locked vault
        let lock_state = if !key_path.exists() {
            VaultLockState::Uninitialized
        } else if !Self::is_current_version(&dir) {
            VaultLockState::NeedsUpgrade
        } else {
            VaultLockState::Locked
        };

        Self {
            lock_state,
            password_input: String::new(),
            confirm_input: String::new(),
            is_setup: false,
            derived_key: None,
            entries: Vec::new(),
            temp_dirs: Vec::new(),
            open_procs: Vec::new(),
            error: None,
            status: None,
            drag_hover: false,
            context_menu_entry: None,
            hidden_apps: Vec::new(),
            hidden_context_menu: None,
        }
    }

    // ── Version / upgrade ─────────────────────────────────────────────────

    /// True if the on-disk vault is the current format: has a .version file equal
    /// to VAULT_FORMAT_VERSION AND a .salt file (v2 requires the per-install salt).
    fn is_current_version(dir: &Path) -> bool {
        let version_ok = fs::read_to_string(dir.join(VERSION_FILE))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .map(|v| v == encryption::VAULT_FORMAT_VERSION)
            .unwrap_or(false);
        version_ok && dir.join(SALT_FILE).exists()
    }

    /// Perform the empty-and-rebuild upgrade: DELETE the entire old vault dir
    /// (the user has been told to back up and empty it first) and return to the
    /// uninitialized state so the next step is a fresh v2 setup.
    ///
    /// We intentionally destroy everything: there is no ciphertext migration, so
    /// nothing half-converted can ever exist. Caller is responsible for having
    /// warned the user. This cannot brick data that the user already removed.
    pub fn confirm_upgrade_wipe(&mut self) -> bool {
        let dir = vault_dir();
        // Best-effort: move aside to a timestamped backup rather than hard-delete,
        // so even a user who ignored the "empty it" warning isn't instantly lost.
        // They can recover the backup dir manually; we never read it automatically.
        let backup = dir.with_file_name(format!(
            "vault.pre-upgrade-backup.{}",
            encryption::unix_now()
        ));
        if dir.exists() {
            if let Err(e) = fs::rename(&dir, &backup) {
                // If rename fails (e.g. cross-device), fall back to recursive delete.
                eprintln!("[VAULT] backup rename failed ({e}); deleting in place");
                if let Err(e2) = fs::remove_dir_all(&dir) {
                    self.error = Some(format!("Could not remove old vault: {e2}"));
                    return false;
                }
            } else {
                self.status = Some(format!(
                    "Old vault backed up to {}. Re-create your vault now.",
                    backup.display()
                ));
            }
        }
        // Fresh start: next state is first-run setup (no key file present).
        self.lock_state = VaultLockState::Uninitialized;
        self.derived_key = None;
        self.entries.clear();
        self.hidden_apps.clear();
        self.error = None;
        self.wipe_inputs();
        true
    }

    /// Dead man's switch — permanent, unrecoverable destruction.
    ///
    /// Unlike confirm_upgrade_wipe(), this does NOT move the vault aside to a
    /// backup. It HARD-DELETES the entire vault directory: .key, .salt,
    /// .version, every encrypted blob + metadata, and the hidden_apps/ tree.
    /// There is intentionally no backup and no recovery — destruction is the
    /// whole point. This is the "Forgot password" action: anyone (the owner, or
    /// someone at the machine after the owner is gone) can wipe the vault back
    /// to first-run state without knowing the password.
    ///
    /// NOTE: this unlinks files; it does not scrub the underlying flash. The
    /// encrypted blobs are ChaCha20-Poly1305 ciphertext, so even if recovered
    /// from raw storage they are useless without the password. For a normal
    /// finder the data is simply gone.
    pub fn forget_and_destroy(&mut self) -> bool {
        // Wipe any open plaintext temp files first.
        self.cleanup_temp();

        let dir = vault_dir();
        if dir.exists() {
            if let Err(e) = fs::remove_dir_all(&dir) {
                self.error = Some(format!("Could not destroy vault: {e}"));
                return false;
            }
        }

        // Back to first-run setup (no .key present).
        self.lock_state = VaultLockState::Uninitialized;
        self.derived_key = None;
        self.entries.clear();
        self.hidden_apps.clear();
        self.temp_dirs.clear();
        self.open_procs.clear();
        self.error = None;
        self.status = Some("Vault destroyed. Set a new password to start fresh.".into());
        self.wipe_inputs();
        true
    }

    /// Read the per-install salt; v2 vaults always have one.
    fn read_salt(dir: &Path) -> Result<Vec<u8>, String> {
        fs::read(dir.join(SALT_FILE)).map_err(|e| format!("Could not read salt: {e}"))
    }

    // ── Setup (first launch / post-upgrade) ───────────────────────────────

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

        // Store password-verification hash (.key).
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

        // Generate + store the per-install v2 salt.
        let salt = encryption::generate_salt();
        if let Err(e) = fs::write(dir.join(SALT_FILE), &salt) {
            self.error = Some(format!("Could not save salt: {e}"));
            return false;
        }

        // Stamp the format version.
        if let Err(e) = fs::write(
            dir.join(VERSION_FILE),
            encryption::VAULT_FORMAT_VERSION.to_string(),
        ) {
            self.error = Some(format!("Could not write version: {e}"));
            return false;
        }

        // Derive the v2 key from password + the salt we just stored.
        match encryption::derive_key_v2(&self.password_input, &salt) {
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
        let dir = vault_dir();
        let key_path = dir.join(KEY_FILE);

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

        // v2 unlock: derive from password + the stored per-install salt.
        let salt = match Self::read_salt(&dir) {
            Ok(s) => s,
            Err(e) => {
                self.error = Some(e);
                return false;
            }
        };

        match encryption::derive_key_v2(&self.password_input, &salt) {
            Ok(key) => {
                self.derived_key = Some(key);
                self.lock_state = VaultLockState::Unlocked;
                self.error = None;
                self.wipe_inputs();
                self.load_entries();
                self.hidden_apps = hidden_apps::load_all(self.key_bytes().unwrap_or(&[]));
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
        // Wipe any open plaintext temp files immediately on lock.
        self.cleanup_temp();

        // NeedsUpgrade is sticky. An old-format vault can't be locked into a
        // usable state — it has to stay in the upgrade flow until the user
        // completes it (confirm_upgrade_wipe). reset_to_default() calls lock()
        // on every show; without this guard that silently downgrades
        // NeedsUpgrade -> Locked and routes the user to an unlock prompt for a
        // vault they can't unlock, so the upgrade screen never appears.
        if self.lock_state == VaultLockState::NeedsUpgrade {
            self.wipe_inputs();
            return;
        }

        self.derived_key = None;
        self.entries.clear();
        self.hidden_apps.clear();
        self.lock_state = VaultLockState::Locked;
        self.error = None;
        self.status = None;
        self.wipe_inputs();
    }

    // ── Add file ──────────────────────────────────────────────────────────

    /// Encrypts `source` into the vault then removes the original.
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

        if let Err(e) = fs::remove_file(source) {
            eprintln!(
                "Could not remove original ({}): {e} — needs pkexec",
                source.display()
            );
            self.entries.push(VaultEntry { id, meta });
            return Err(format!("NEEDS_PKEXEC:{}", source.display()));
        }

        self.entries.push(VaultEntry { id, meta });
        self.status = Some(format!("'{original_name}' added to vault."));
        Ok(())
    }

    #[allow(dead_code)] // issue #3
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
            Err(format!("pkexec rm failed with code {:?}", status.code()))
        }
    }

    // ── Open file ─────────────────────────────────────────────────────────

    /// Decrypt a vaulted file to a RAM-backed tmpfs path (NOT persistent disk),
    /// with owner-only (0600) permissions, then open it. The child process is
    /// tracked so the plaintext is wiped the instant the viewer/editor exits
    /// (see reap_finished_opens), and unconditionally on lock / launcher close.
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

        // tmpfs-backed dir, owner-only.
        let tmp_dir = runtime_tmp_base()
            .join(format!("soulless-vault-{}", encryption::random_id()));
        fs::create_dir_all(&tmp_dir)
            .map_err(|e| format!("Could not create temp dir: {e}"))?;
        set_dir_private(&tmp_dir);

        let tmp_file = tmp_dir.join(&entry.meta.original_name);
        fs::write(&tmp_file, &plaintext)
            .map_err(|e| format!("Could not write temp file: {e}"))?;
        set_file_private(&tmp_file);

        // Launch and TRACK the child so we can wipe as soon as it exits.
        let child = std::process::Command::new("xdg-open")
            .arg(&tmp_file)
            .spawn()
            .map_err(|e| format!("Could not open file: {e}"))?;

        self.temp_dirs.push(tmp_dir.clone());
        self.open_procs.push((child, tmp_dir));
        Ok(())
    }

    /// Wipe temp dirs whose opening process has exited. Call periodically (e.g.
    /// on each launcher tick) so plaintext doesn't linger after the user closes
    /// the viewer. Non-blocking: uses try_wait.
    pub fn reap_finished_opens(&mut self) {
        let mut still_open = Vec::new();
        for (mut child, tmp) in std::mem::take(&mut self.open_procs) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process exited → wipe its plaintext now.
                    let _ = fs::remove_dir_all(&tmp);
                    self.temp_dirs.retain(|d| d != &tmp);
                }
                Ok(None) => still_open.push((child, tmp)), // still running
                Err(_) => {
                    // Can't tell; keep it and clean up at lock/close.
                    still_open.push((child, tmp));
                }
            }
        }
        self.open_procs = still_open;
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

        // Don't clobber: if a file of this name exists, suffix it.
        let mut dest = downloads.join(&entry.meta.original_name);
        if dest.exists() {
            let stem = Path::new(&entry.meta.original_name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file")
                .to_string();
            let ext = Path::new(&entry.meta.original_name)
                .extension()
                .and_then(|s| s.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();
            dest = downloads.join(format!("{stem}.{}{ext}", encryption::unix_now()));
        }

        fs::write(&dest, &plaintext)
            .map_err(|e| format!("Could not export file: {e}"))?;
        self.status = Some(format!("Exported to {}.", dest.display()));
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

    // ── Hidden apps ───────────────────────────────────────────────────────
    pub fn hide_app(&mut self, desktop_path: &Path) -> Result<(), String> {
        let key = self.key_bytes()?;
        let app = hidden_apps::hide(key, desktop_path)?;
        let name = app.meta.name.clone();
        self.hidden_apps.push(app);
        self.status = Some(format!("'{name}' hidden in vault."));
        Ok(())
    }

    pub fn unhide_app(&mut self, id: &str) -> Result<(), String> {
        let key = self.key_bytes()?;
        if let Some(pos) = self.hidden_apps.iter().position(|a| a.id == id) {
            hidden_apps::unhide(key, &self.hidden_apps[pos])?;
            let app = self.hidden_apps.remove(pos);
            self.status = Some(format!("'{}' restored.", app.meta.name));
        }
        Ok(())
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
            let Ok(meta) = serde_json::from_str::<VaultFileMeta>(&json) else {
                continue;
            };
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let enc_path = dir.join(format!("{id}{ENC_EXT}"));
            if !enc_path.exists() {
                continue;
            }
            self.entries.push(VaultEntry { id, meta });
        }
        self.entries.sort_by(|a, b| b.meta.added_at.cmp(&a.meta.added_at));
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

// ── Unix permission helpers (owner-only) ──────────────────────────────────────
// 0600 file / 0700 dir so other local users can't read decrypted plaintext while
// it's open. No-ops on non-unix (the project is Linux-only, but keep it portable).

#[cfg(unix)]
fn set_file_private(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}
#[cfg(unix)]
fn set_dir_private(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o700));
}
#[cfg(not(unix))]
fn set_file_private(_path: &Path) {}
#[cfg(not(unix))]
fn set_dir_private(_path: &Path) {}