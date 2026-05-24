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
