// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/vault/encryption.rs
// All cryptographic operations for the vault.
//
// HARDENING (v2):
//   - Per-install RANDOM salt for key derivation (was a hardcoded in-source salt,
//     which let one precomputation attack every install). The salt is generated
//     once at vault creation and stored in the vault dir. Same salt every session
//     for THIS install, unique across installs.
//   - Deliberately EXPENSIVE Argon2id params (high memory cost) so brute-forcing a
//     stolen vault blob is punishing, not cheap. (Was Argon2::default().)
//
// The legacy v1 derivation (hardcoded salt, default params) is KEPT as
// `derive_key_v1_legacy` so an old vault can still be unlocked once, for the
// empty-and-rebuild upgrade flow. It is never used for new encryption.

use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use secrecy::SecretVec;
use std::path::Path;

pub(super) const NONCE_SIZE: usize = 12;

// ── Vault format version ──────────────────────────────────────────────────────
// Bump when the on-disk crypto format changes in a way that requires upgrade.
pub(super) const VAULT_FORMAT_VERSION: u32 = 2;

// ── Argon2id parameters (v2) ──────────────────────────────────────────────────
// Deliberately heavy. m_cost is in KiB. 256 MiB memory, 4 passes, 1 lane.
// Tune if startup-unlock feels too slow on low-end hardware, but do NOT drop
// these to defaults — the whole point is to make offline brute force expensive.
const ARGON2_M_COST_KIB: u32 = 256 * 1024; // 256 MiB
const ARGON2_T_COST: u32 = 4; // iterations
const ARGON2_P_COST: u32 = 1; // parallelism (lanes)

/// Build the hardened Argon2id instance used for v2 key derivation.
fn argon2_v2() -> Result<Argon2<'static>, String> {
    let params = Params::new(
        ARGON2_M_COST_KIB,
        ARGON2_T_COST,
        ARGON2_P_COST,
        Some(32), // output length: 32-byte key
    )
    .map_err(|e| format!("Argon2 params error: {e}"))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

// ── Password hashing (for the .key verification file) ─────────────────────────
// Unchanged interface: stores a PHC-string hash with its own embedded random salt.
// (This was already fine — the weakness was in derive_key, not here.)

pub(super) fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon2_v2()?;
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub(super) fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed = PasswordHash::new(hash).map_err(|e| e.to_string())?;
    // Verify with a default Argon2; the PHC string carries its own params, so this
    // validates regardless of which params produced it (v1 default OR v2 heavy).
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

// ── Key derivation ────────────────────────────────────────────────────────────

/// LEGACY v1 derivation — hardcoded salt, DEFAULT Argon2 params.
/// KEPT ONLY so an existing v1 vault can be unlocked once during the
/// empty-and-rebuild upgrade. NEVER used to encrypt new data.
// #29: legacy v1 KDF kept for old-vault recovery; unused until/if a "read old
// vault to export" path lands.
#[allow(dead_code)]
pub(super) fn derive_key_v1_legacy(password: &str) -> Result<SecretVec<u8>, String> {
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

/// v2 derivation — per-install random `salt` + heavy Argon2id params.
/// `salt` comes from the vault's stored salt file (see mod.rs).
pub(super) fn derive_key_v2(password: &str, salt: &[u8]) -> Result<SecretVec<u8>, String> {
    let mut key = vec![0u8; 32];
    argon2_v2()?
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| e.to_string())?;
    Ok(SecretVec::new(key))
}

/// Generate a fresh per-install salt (32 random bytes). Called once at v2 vault
/// creation; the result is persisted and reused every unlock for this install.
pub(super) fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

// ── Symmetric encryption (unchanged — this part was already sound) ────────────

/// Encrypts plaintext with ChaCha20-Poly1305.
/// Output format: [12-byte random nonce][ciphertext+tag]
pub(super) fn encrypt_data(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
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
pub(super) fn decrypt_data(key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
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

// ── Misc helpers (unchanged) ──────────────────────────────────────────────────

pub(super) fn random_id() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub(super) fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Extension-based MIME type detection — no external crate needed.
pub(super) fn mime_from_path(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "mp4" | "m4v" => "video/mp4",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "flv" => "video/x-flv",
        "wmv" => "video/x-ms-wmv",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "tiff" | "tif" => "image/tiff",
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "html" | "htm" => "text/html",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "xz" => "application/x-xz",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",
        _ => "application/octet-stream",
    }
    .to_string()
}