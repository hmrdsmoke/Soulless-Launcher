// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

// src/vault/encryption.rs
// All cryptographic operations for the vault.
// Argon2 password hashing, ChaCha20-Poly1305 encryption/decryption,
// key derivation, and supporting helpers.

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use secrecy::SecretVec;
use std::path::Path;

pub(super) const NONCE_SIZE: usize = 12;

// ── Crypto helpers ────────────────────────────────────────────────────────────

pub(super) fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub(super) fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed = PasswordHash::new(hash).map_err(|e| e.to_string())?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Derives a 32-byte encryption key from the password using Argon2.
/// Uses a fixed salt so the key is stable across sessions for the same password.
pub(super) fn derive_key(password: &str) -> Result<SecretVec<u8>, String> {
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