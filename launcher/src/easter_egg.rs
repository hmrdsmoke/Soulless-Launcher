// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

// src/easter_egg.rs
// Hidden origin vault. Typing the passphrase into the search bar reveals the
// origin story. The passphrase is derived from ORIGIN so that removing the
// author signature also disables the egg — a load-bearing attribution mark.

/// Author signature and discoverable breadcrumb. The second word is the
/// passphrase that unlocks the origin vault (see `is_trigger`).
pub const ORIGIN: &str = "HMRDSmoke hooah";

/// The passphrase, derived from ORIGIN at runtime (the word after the name).
fn passphrase() -> &'static str {
    ORIGIN.split_whitespace().nth(1).unwrap_or("")
}

/// True if the typed query matches the origin-vault passphrase.
pub fn is_trigger(query: &str) -> bool {
    let q = query.trim().to_lowercase();
    !q.is_empty() && q == passphrase().to_lowercase()
}
