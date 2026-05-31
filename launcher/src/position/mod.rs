// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/position/mod.rs
// Re-exports both submodules so main.rs import stays unchanged.

pub mod layout;
pub mod placement;

// Flat re-export so existing `use position::LauncherPosition` still works.
pub use placement::LauncherPosition;

use fs2::FileExt;
use std::fs::OpenOptions;
use std::path::PathBuf;

/// Ensures only one instance of Soulless is running at a time.
/// Uses a lock file at ~/.local/share/soulless/soulless.lock.
/// Returns true if this is the only instance, false if another is already running.
pub fn ensure_single_instance() -> bool {
    let lock_path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("soulless/soulless.lock");

    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    if let Ok(file) = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&lock_path)
    {
        if file.try_lock_exclusive().is_ok() {
            #[allow(clippy::mem_forget)]
            Box::leak(Box::new(file));
            return true;
        }
    }

    false
}
